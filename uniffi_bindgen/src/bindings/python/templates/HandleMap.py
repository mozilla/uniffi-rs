# Initial value and increment amount for handles. 
# These ensure that Python-generated handles always have the lowest bit set
_UNIFFI_HANDLEMAP_INITIAL = 1
_UNIFFI_HANDLEMAP_DELTA = 2

class _UniffiHandleMap:
    """
    A map where inserting, getting and removing data is synchronized with a lock.
    """

    def __init__(self):
        # type Handle = int
        self._map = {}  # type: Dict[Handle, Any]
        self._lock = threading.Lock()
        self._counter = _UNIFFI_HANDLEMAP_INITIAL

    def insert(self, obj):
        with self._lock:
            return self._insert(obj)

    """Low-level insert, this assumes `self._lock` is held."""
    def _insert(self, obj):
        handle = self._counter
        self._counter += _UNIFFI_HANDLEMAP_DELTA
        self._map[handle] = obj
        return handle

    def get(self, handle):
        try:
            with self._lock:
                return self._map[handle]
        except KeyError:
            raise InternalError(f"_UniffiHandleMap.get: Invalid handle {handle}")

    def clone(self, handle):
        try:
            with self._lock:
                obj = self._map[handle]
                return self._insert(obj)
        except KeyError:
            raise InternalError(f"_UniffiHandleMap.clone: Invalid handle {handle}")

    def remove(self, handle):
        try:
            with self._lock:
                return self._map.pop(handle)
        except KeyError:
            raise InternalError(f"_UniffiHandleMap.remove: Invalid handle: {handle}")

    def __len__(self):
        return len(self._map)
