
FUTURE_WAKER_T = ctypes.CFUNCTYPE(ctypes.c_uint8)

class RustFuture(ctypes.Structure):
    _fields_ = [
        ("_padding", ctypes.c_void_p),
    ]

class FuturePoll(enum.Enum):
    PENDING = 0
    DONE = 1

class Future:
    def __init__(self, future: any):
        self._asyncio_future_blocking = False
        self._loop = asyncio.get_event_loop()
        self._state = FuturePoll.PENDING
        self._result = None
        self._future = future
        self._waker = None
        self._ffi_waker = None
        self._callbacks = []

        def waker():
            def async_waker():
                state, self._result = (self._future)()

                if state == FuturePoll.DONE:
                    self.set_result(self._result)

            self._loop.call_soon_threadsafe(async_waker)

            return 0

        self._waker = waker
        self._ffi_waker = FUTURE_WAKER_T(waker)

    def _future_waker(self) -> any:
        return self._waker

    def _future_ffi_waker(self) -> FUTURE_WAKER_T:
        return self._ffi_waker

    def done(self) -> bool:
        return self._state == FuturePoll.DONE

    def result(self) -> any:
        if self._state != FuturePoll.DONE:
            raise RuntimeError('Result is not ready')

        return self._result

    def set_result(self, result: any):
        if self._state != FuturePoll.PENDING:
            raise RuntimeError('This future has already been resolved')

        self._result = result
        self._state = FuturePoll.DONE
        self.__schedule_callbacks()

    def __schedule_callbacks(self):
        callbacks = self._callbacks[:]

        if not callbacks:
            return

        self._callbacks[:] = []

        for callback, context in callbacks:
            self._loop.call_soon_threadsafe(callback, self, context=context)

    def add_done_callback(self, callback, *, context=None):
        if self._state != FuturePoll.PENDING:
            self._loop.call_soon_threadsafe(callback, self, context=context)
        else:
            if context is None:
                context = contextvars.copy_context()

            self._callbacks.append((callback, context))

    def remove_done_callback(self, callback):
        filtered_callbacks = [(other_callback, context)
                              for (other_callback, context) in self._callbacks
                              if other_callback != callback]
        removed_count = len(self._callbacks) - len(filtered_callbacks)

        if removed_count:
            self._callbacks[:] = filtered_callbacks

        return removed_count

    def cancel(self, msg=None):
        pass # TODO

    def __await__(self):
        if not self.done():
            self._asyncio_future_blocking = True
            yield self

        if not self.done():
            raise RuntimeError('await wasn\'t used with future')

        return self.result()

    __iter__ = __await__

