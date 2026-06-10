# RustFuture poll codes
UNIFFI_RUST_FUTURE_POLL_READY = 0
UNIFFI_RUST_FUTURE_POLL_WAKE = 1

# Handle map for storing write-end IO objects used by the continuation callbacks.
UNIFFI_ASYNC_HANDLE_MAP = UniffiHandleMap.new

# Continuation callback for async functions.
# Called by Rust when the future is ready to make progress.
# Writes the poll code to the pipe so the waiting thread/fiber can continue.
#
# Exceptions must never escape this Proc.
# Rust invokes it directly via FFI, so any unhandled Ruby exception would be caught
# by Ruby-FFI, which swallows it (prints a warning) and returns a garbage value to Rust.
# The poll code Rust receives would be invalid, causing the Ruby polling loop to
# never see POLL_READY and block forever.
UNIFFI_CONTINUATION_CALLBACK = Proc.new do |data, poll_code|
  begin
    wr = UNIFFI_ASYNC_HANDLE_MAP.get data

    # This can only be blocking if pipe if full. Rust will never try to call continuation
    # callback more than once (so will not fill it up), hence this call should never block.
    wr.putc poll_code
  rescue Exception
    # Swallow exception. A leak or a hang is better than a hard VM segfault.
  end
end

# Poll a Rust future to completion.
#
# This works both with and without a Fiber::Scheduler:
# - Without scheduler: wait_readable blocks with a timeout, releasing the GVL
#   so the callback can fire from foreign threads (works around MRI limitation
#   where rb_thread_call_with_gvl cannot wake up threads in indefinite sleep).
# - With scheduler: wait_readable hooks into io_wait, yielding the fiber.
#
# cancel_fn is called in the ensure block when exception interrupts an in-flight poll.
# This guarantees Rust fires the continuation callback so the handle-map entry is released
# and the pipe is drained before we free the future.
def self.uniffi_rust_call_async(rust_future, poll_fn, cancel_fn, complete_fn, free_fn, lift_func, error_ffi_converter)
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

      # Blocks until the continuation callback writes to the pipe.
      # Releases the GVL so the callback can fire from foreign threads.
      # With a Fiber::Scheduler, this hooks into io_wait for non-blocking concurrency.
      rd.wait_readable
      poll_code = rd.getbyte
      poll_in_flight = false

      break if poll_code == UNIFFI_RUST_FUTURE_POLL_READY
    end

    result = if error_ffi_converter.nil?
      ::{{ ci.namespace()|class_name_rb }}.rust_call(complete_fn, rust_future)
    else
      ::{{ ci.namespace()|class_name_rb }}.rust_call_with_error(error_ffi_converter, complete_fn, rust_future)
    end

    lift_func.call(result)
  ensure
    # Defer any further Thread#raise / Timeout during cleanup so all steps execute
    # automatically. Without this, a second raise during wait_readable(0.5) would skip
    # handle-map removal, pipe close, and free_fn - leaking FDs and Rust memory.
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

# One-shot claim flag: the first caller to `claim!` wins; all subsequent callers
# are no-ops. Used to enforce the at-most-once contract on uniffi_future_callback.
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

# Called by Rust when the foreign future is dropped (i.e. canceled or completed successfully).
# Raises UniffiInternalCancelled in the worker thread so make_call can exit early,
# but only if the thread hasn't already completed and claimed the once flag.
# Stored as a constant to prevent GC from collecting the Proc while Rust holds the pointer.
UNIFFI_FOREIGN_FUTURE_DROPPED_CALLBACK = Proc.new do |handle|
  thread, once = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.remove handle
  thread.raise(UniffiInternalCancelled, 'Future was canceled') if once.claim! && thread&.alive?
end

# Execute a foreign async callback method in a background thread.
# Enforces the at-most-once guarantee on handle_success / handle_error: whichever
# fires first (normal completion or Rust-side drop) suppresses the other.
def self.uniffi_trait_interface_call_async(make_call, uniffi_out_dropped_callback, handle_success, handle_error, error_type = nil, lower_error = nil)
  once = UniffiOnceFlag.new

  thread = Thread.new do
    begin
      # Phase 1: run the user's async method.
      # UniffiInternalCancelled exits silently. Other exceptions are forwarded as errors.
      # handle_success is intentionally called outside this rescue so exceptions from it
      # cannot re-enter handle_error (which would be a double-call on the Rust sender).
      begin
        result = make_call.call
      rescue UniffiInternalCancelled
        next
      rescue Exception => e # We have to catch all errors to prevent Rust future from hanging forever.
        next unless once.claim!

        if !error_type.nil? && ::{{ ci.namespace()|class_name_rb }}.uniffi_is_error_type?(e, error_type)
          handle_error.call(UNIFFI_CALLBACK_ERROR, lower_error.call(e))
        else
          handle_error.call(UNIFFI_CALLBACK_UNEXPECTED_ERROR, {{ "e.inspect"|lower_rb(&Type::String, config) }})
        end
        next
      end

      # Phase 2: deliver the result to Rust. Skipped if dropped_callback already fired.
      handle_success.call(result) if once.claim!
    rescue UniffiInternalCancelled
      # Thread#raise landed between phases or during Phase 2 - silently exit.
      # Rust already dropped the future (that's why dropped_callback fired), so no response needed.
    rescue Exception => e
      # handle_success/handle_error/lower_error raised - send a generic error so Rust doesn't hang.
      # once was already claimed, so only attempt this if we can still claim (e.g. lowering failed
      # before handle_error was called due to short-circuit evaluation).
      begin
        handle_error.call(UNIFFI_CALLBACK_UNEXPECTED_ERROR, {{ "e.inspect"|lower_rb(&Type::String, config) }})
      rescue Exception
        # If even this fails, Rust will hang. Nothing more we can do.
      end
    end
  end

  # Note: the thread may have already completed by this point, but that's safe.
  # Rust cannot invoke dropped_callback until this function returns.
  # possesses the ForeignFuture struct we're populating here.
  handle = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.insert([thread, once])
  uniffi_out_dropped_callback[:handle] = handle
  uniffi_out_dropped_callback[:free] = UNIFFI_FOREIGN_FUTURE_DROPPED_CALLBACK
end
{%- endif %}
