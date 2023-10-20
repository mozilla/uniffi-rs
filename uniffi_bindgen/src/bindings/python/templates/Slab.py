UniffiHandle = typing.NewType('UniffiHandle', int)

# TODO: it would be nice to make this a generic class, however let's wait until python 3.11 is the
# minimum version, so we can do that without having to add a `TypeVar` to the top-level namespace.
class UniffiSlab:
    def __init__(self):
        self.slab_handle = _UniffiLib.{{ ci.ffi_slab_new().name() }}()
        # We don't need a lock around the items list, since `list.append()` is atomic:
        #   * Python FAQ: https://docs.python.org/2/faq/library.html#what-kinds-of-global-value-mutation-are-thread-safe
        #   * PEP 703 – Making the Global Interpreter Lock Optional in CPython includes a section
        #     specifying that `list.append()` should stay atomic: https://peps.python.org/pep-0703/#container-thread-safety
        self.items = []

    def __del__(self):
        _UniffiLib.{{ ci.ffi_slab_free().name() }}(self.slab_handle)

    def _index(self, handle: UniffiHandle) -> int:
        return handle & 0xFFFF

    def insert(self, value: object) -> UniffiHandle:
        handle = _UniffiLib.{{ ci.ffi_slab_insert().name() }}(self.slab_handle)
        if handle < 0:
            raise InternalError("Slab insert error")
        index = self._index(handle)
        while len(self.items) <= index:
            self.items.append(None)
        self.items[index] = value
        return handle

    def get(self, handle: UniffiHandle) -> object:
        result = _UniffiLib.{{ ci.ffi_slab_check_handle().name() }}(self.slab_handle, handle)
        if result < 0:
            raise InternalError("Slab get error")
        return self.items[self._index(handle)]

    def remove(self, handle: UniffiHandle) -> object:
        result = _UniffiLib.{{ ci.ffi_slab_dec_ref().name() }}(self.slab_handle, handle)
        if result < 0:
            raise InternalError("Slab remove error")
        index = self._index(handle)
        value = self.items[index]
        if result == 1:
            self.items[index] = None
        return value
