# RustFuture poll codes
UNIFFI_RUST_FUTURE_POLL_READY = 0
UNIFFI_RUST_FUTURE_POLL_WAKE = 1

# Handle map for storing write-end IO objects used by the continuation callbacks.
UNIFFI_ASYNC_HANDLE_MAP = UniffiHandleMap.new

# Continuation callback for async functions.
# Called by Rust when the future is ready to make progress.
# Writes the poll code to the pipe so the waiting thread/fiber can continue.
UNIFFI_CONTINUATION_CALLBACK = Proc.new do |data, poll_code|
  wr = UNIFFI_ASYNC_HANDLE_MAP.remove(data)
  wr.write([poll_code].pack('C'))
  wr.close
end

# Poll a Rust future to completion.
#
# This works both with and without a Fiber::Scheduler:
# - Without scheduler: wait_readable blocks with a timeout, releasing the GVL
#   so the callback can fire from foreign threads (works around MRI limitation
#   where rb_thread_call_with_gvl cannot wake up threads in indefinite sleep).
# - With scheduler: wait_readable hooks into io_wait, yielding the fiber.
def self.uniffi_rust_call_async(rust_future, poll_fn, complete_fn, free_fn, lift_func, error_ffi_converter)
  begin
    loop do
      rd, wr = IO.pipe
      handle = UNIFFI_ASYNC_HANDLE_MAP.insert(wr)
      UniFFILib.public_send(poll_fn, rust_future, UNIFFI_CONTINUATION_CALLBACK, handle)

      # wait_readable with a timeout ensures periodic thread wakeups and maps cleanly to
      # Fiber::Scheduler#io_wait for non-blocking fiber concurrency.
      nil until rd.wait_readable(0.01)
      poll_code = rd.read(1).unpack1('C')
      rd.close

      break if poll_code == UNIFFI_RUST_FUTURE_POLL_READY
    end

    result = if error_ffi_converter.nil?
      ::{{ ci.namespace()|class_name_rb }}.rust_call(complete_fn, rust_future)
    else
      ::{{ ci.namespace()|class_name_rb }}.rust_call_with_error(error_ffi_converter, complete_fn, rust_future)
    end

    lift_func.call(result)
  ensure
    UniFFILib.public_send(free_fn, rust_future)
  end
end

{%- if ci.has_async_callback_interface_definition() %}
# Exception raised when a foreign future is canceled.
class UniffiCancelled < RuntimeError; end

# Handle map for storing Threads executing foreign async callbacks.
UNIFFI_FOREIGN_FUTURE_THREAD_MAP = UniffiHandleMap.new

# Dropped callback: cancels the executing thread when rust drops the future
UNIFFI_FOREIGN_FUTURE_DROPPED_CALLBACK = Proc.new do |handle|
  thread = UNIFFI_FOREIGN_FUTURE_THREAD_MAP.remove(handle)
  thread.raise(UniffiCancelled, "Future was canceled") if thread&.alive?
end

# Execute a foreign async callback method in a background thread.
# Calls handle_success or handle_error exactly once when done.
# If error_type is provided, known errors of that type are reported as UNIFFI_CALLBACK_ERROR;
# all other exceptions are reported as UNIFFI_CALLBACK_UNEXPECTED_ERROR.
def self.uniffi_trait_interface_call_async(make_call, uniffi_out_dropped_callback, handle_success, handle_error, error_type = nil, lower_error = nil)
  thread = Thread.new do
    begin
      result = make_call.call
      handle_success.call(result)
    rescue UniffiCancelled
      # Future was canceled, do nothing (Rust already dropped the future).
    rescue StandardError => e
      if !error_type.nil? && ::{{ ci.namespace()|class_name_rb }}.uniffi_is_error_type?(e, error_type)
        handle_error.call(UNIFFI_CALLBACK_ERROR, lower_error.call(e))
      else
        handle_error.call(
          UNIFFI_CALLBACK_UNEXPECTED_ERROR,
          {{ "e.inspect"|lower_rb(&Type::String, config) }}
        )
      end
    end
  end

  handle = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.insert(thread)
  uniffi_out_dropped_callback[:handle] = handle
  uniffi_out_dropped_callback[:free] = UNIFFI_FOREIGN_FUTURE_DROPPED_CALLBACK
end
{%- endif %}
