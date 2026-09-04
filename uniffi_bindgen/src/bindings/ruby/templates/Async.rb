# RustFuture poll codes
UNIFFI_RUST_FUTURE_POLL_READY = 0
UNIFFI_RUST_FUTURE_POLL_WAKE = 1

# Handle map for storing write-end IO objects used by the continuation callbacks.
UNIFFI_ASYNC_HANDLE_MAP = UniffiHandleMap.new

# Continuation callback for async functions.
# Called by Rust when the future is ready to make progress.
# Writes the poll code (an argument from Rust; this Proc returns void) to the
# pipe so the waiting thread/fiber can continue.
#
# Exceptions must never escape this Proc.
# FFI 1.17 rescues Exception around the callback and zeros the C return
# (void here, so unused). If this ran during a Ruby-initiated FFI call, the
# exception is re-raised after that call returns; if it ran via the async
# dispatcher, it is discarded. Either way putc never happens and the waiter
# blocks forever.
UNIFFI_CONTINUATION_CALLBACK = Proc.new do |data, poll_code|
  begin
    wr = UNIFFI_ASYNC_HANDLE_MAP.get data

    # putc blocks only if the pipe is full. Each rust_future_poll invokes the
    # continuation exactly once, and this loop reads that byte before polling
    # again, so the pipe holds at most one unread byte.
    wr.putc poll_code
  rescue Exception
    # Swallow exception. A leak or a hang is better than a hard VM segfault.
  end
end

# Poll a Rust future to completion.
#
# One handle and one pipe are reused for the whole poll loop. The continuation
# looks the write end up with `get` (it does not remove the handle). ensure
# drains the pipe, then removes the handle, closes the FDs, and frees the future.
#
# This works both with and without a Fiber::Scheduler:
# - Without scheduler: wait_readable waits indefinitely in rb_thread_io_wait,
#   which releases the GVL. That lets another Ruby thread run the continuation
#   and putc, which wakes this wait.
# - With scheduler: wait_readable calls Fiber::Scheduler#io_wait and yields
#   the fiber. GVL is released later, when the scheduler blocks in select.
#
# When an exception interrupts an in-flight poll, ensure calls cancel_fn so
# Rust invokes the stored continuation (if any). That unblocks wait_readable
# so the pipe can be drained while the handle and write end are still valid.
# Cancel does not itself release the handle. If poll was never sent, cancel
# writes nothing and wait_readable(0.5) times out.
def self.uniffi_rust_call_async(rust_future, poll_fn, cancel_fn, complete_fn, free_fn, lift_func, error_reader)
  rd = wr = nil
  handle = nil
  poll_in_flight = false

  begin
    rd, wr = IO.pipe
    wr.sync = true # avoid buffering and delayed read for rd.wait_readable
    handle = UNIFFI_ASYNC_HANDLE_MAP.insert wr

    loop do
      poll_in_flight = true
      UniFFILib.public_send(poll_fn, rust_future, UNIFFI_CONTINUATION_CALLBACK, handle)

      # Wait until the pipe is readable. The continuation may already have
      # written during poll (Ready/Wake invoked synchronously); then this
      # returns immediately. Without a scheduler this is a GVL-releasing
      # blocking wait; with one it is scheduler.io_wait.
      rd.wait_readable
      poll_code = rd.getbyte
      poll_in_flight = false

      break if poll_code == UNIFFI_RUST_FUTURE_POLL_READY
    end

    result = if error_reader.nil?
      ::{{ self.module_name() }}.rust_call(complete_fn, rust_future)
    else
      ::{{ self.module_name() }}.rust_call_with_error(error_reader, complete_fn, rust_future)
    end

    lift_func.call(result)
  ensure
    # Defer Thread#raise / Timeout until this block returns so every cleanup
    # step runs. Without this, a second raise during wait_readable(0.5) would
    # skip handle-map removal, pipe close, and free_fn.
    Thread.handle_interrupt(Exception => :never) do
      if poll_in_flight
        # An exception interrupted an in-flight poll. Cancel and drain the byte
        # the continuation callback will write so we don't leak the pipe.
        UniFFILib.public_send(cancel_fn, rust_future)
        # rd.wait_readable may time out and return nil if the poll was never actually sent
        # (raise landed between poll_in_flight=true and the FFI call).
        if rd.wait_readable(0.5)
          rd.getbyte
        end
      end

      # Remove handle first so any late-firing callback's `get` raises (swallowed by rescue).
      UNIFFI_ASYNC_HANDLE_MAP.remove(handle) rescue nil if handle
      rd&.close rescue nil
      wr&.close rescue nil
      UniFFILib.public_send(free_fn, rust_future)
    end
  end
end

{%- if ci.has_async_callback_interface_definition() %}
# Exception raised when a foreign future is canceled.
class UniffiInternalCancelled < RuntimeError; end

# User callback that raises it will be considered a Rust-side cancellation.
private_constant :UniffiInternalCancelled

# Handle map for storing Threads executing foreign async callbacks.
UNIFFI_FOREIGN_FUTURE_HANDLE_MAP = UniffiHandleMap.new

# One-shot claim flag: the first caller to `claim!` wins; later callers get false.
# Used so handle_success / handle_error and the dropped callback cannot both
# complete the Rust future.
class UniffiOnceFlag
  def initialize
    @mutex = Mutex.new
    @claimed = false
  end

  # Returns true if this caller won the race (first to claim), false otherwise.
  def claim!
    @mutex.synchronize do
      first = !@claimed
      @claimed = true
      first
    end
  end
end

# Called by Rust when the foreign future is dropped (cancel, success, or error).
# Claims the once flag and raises UniffiInternalCancelled in the worker if the
# thread is still running, so make_call can exit without completing.
# Stored as a constant to prevent GC from collecting the Proc while Rust holds the pointer.
UNIFFI_FOREIGN_FUTURE_DROPPED_CALLBACK = Proc.new do |handle|
  thread, once = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.remove handle
  thread.raise(UniffiInternalCancelled, 'Future was canceled') if once.claim! && thread&.alive?
end

# Execute a foreign async callback method in a background thread.
# Enforces the at-most-once contract on handle_success / handle_error.
# Rust-side drop claims the once flag so the worker will not complete;
# the worker claims it before delivering a result or error.
def self.uniffi_trait_interface_call_async(make_call, uniffi_out_dropped_callback, handle_success, handle_error, error_class = nil, lower_error = nil)
  once = UniffiOnceFlag.new

  thread = Thread.new do
    begin
      # Phase 1: run the user's async method.
      # UniffiInternalCancelled exits silently. Other exceptions are forwarded as errors.
      # handle_success is outside this rescue so a raise from it cannot also
      # run handle_error from this block (a double-call on the Rust sender).
      begin
        result = make_call.call
      rescue UniffiInternalCancelled
        next
      rescue Exception => e # We have to catch all errors to prevent Rust future from hanging forever.
        next unless once.claim!

        if !error_class.nil? && ::{{ self.module_name() }}.uniffi_is_error_type?(e, error_class)
          handle_error.call(UNIFFI_CALLBACK_ERROR, lower_error.call(e))
        else
          handle_error.call(UNIFFI_CALLBACK_UNEXPECTED_ERROR, {{ self.lower_rb("e.inspect", &Type::String)? }})
        end
        next
      end

      # Phase 2: deliver the result to Rust. Skipped if dropped_callback already fired.
      handle_success.call(result) if once.claim!
    rescue UniffiInternalCancelled
      # Thread#raise landed between phases or during Phase 2 - silently exit.
      # Rust already dropped the future (that's why dropped_callback fired), so no response needed.
    rescue Exception => e
      # handle_success, handle_error, or lower_error raised after once was claimed.
      # Retry handle_error so Rust does not hang if the first complete never
      # reached FFI (e.g. lower_error raised while evaluating handle_error's
      # arguments). If handle_success already completed on the Rust side, this
      # is a second call on the sender.
      begin
        handle_error.call(UNIFFI_CALLBACK_UNEXPECTED_ERROR, {{ self.lower_rb("e.inspect", &Type::String)? }})
      rescue Exception
        # If even this fails, Rust will hang. Nothing more we can do.
      end
    end
  end

  # The worker may already have finished; that is safe. Rust only reads
  # uniffi_out_dropped_callback after this function returns, so it cannot
  # invoke dropped_callback until the handle and Proc are stored.
  handle = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.insert([thread, once])
  uniffi_out_dropped_callback[:handle] = handle
  uniffi_out_dropped_callback[:free] = UNIFFI_FOREIGN_FUTURE_DROPPED_CALLBACK
end
{%- endif %}
