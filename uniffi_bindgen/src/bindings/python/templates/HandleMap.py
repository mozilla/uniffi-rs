UniffiHandle = typing.NewType('UniffiHandle', int)

# TODO: it would be nice to make this a generic class, however let's wait until python 3.11 is the
# minimum version, so we can do that without having to add a `TypeVar` to the top-level namespace.
class UniffiHandleMap:
    """
    Manage handles for objects that are passed across the FFI

    See the `uniffi_core::HandleAlloc` trait for the semantics of each method
    """

    # Generates ids that are likely to be unique for each map
    map_id_counter = itertools.count({{ ci.namespace_hash() }})
 
    def __init__(self):
        self._lock = threading.Lock()
        self._map = {}
        # Map ID, shifted into the top 16 bits
        self._map_id = (next(UniffiHandleMap.map_id_counter) & 0xFFFF) << 48
        # Note: Foreign handles are always odd
        self._key_counter = 1

    def _next_key(self) -> int:
        key = self._key_counter
        self._key_counter = (self._key_counter + 2) & 0xFFFF_FFFF_FFFF
        return key

    def _make_handle(self, key: int) -> int:
        return key | self._map_id

    def _key(self, handle: int) -> int:
        if (handle & 0xFFFF_0000_0000_0000) != self._map_id:
            raise InternalError("Handle map ID mismatch")
        return handle & 0xFFFF_FFFF_FFFF
 
    def new_handle(self, obj: object) -> int:
        with self._lock:
            key = self._next_key()
            self._map[key] = obj
            return self._make_handle(key)
 
    def clone_handle(self, handle: int) -> int:
        try:
            with self._lock:
                obj = self._map[self._key(handle)]
                key = self._next_key()
                self._map[key] = obj
                return self._make_handle(key)
        except KeyError:
            raise InternalError("handlemap key error: was the handle used after being freed?")
 
    def get(self, handle: int) -> object:
        try:
            with self._lock:
                return self._map[self._key(handle)]
        except KeyError:
            raise InternalError("handlemap key error: was the handle used after being freed?")
 
    def consume_handle(self, handle: int) -> object:
        try:
            with self._lock:
                return self._map.pop(self._key(handle))
        except KeyError:
            raise InternalError("handlemap key error: was the handle used after being freed?")

def uniffi_handle_is_from_rust(handle: int) -> bool:
    return (handle & 1) == 0
