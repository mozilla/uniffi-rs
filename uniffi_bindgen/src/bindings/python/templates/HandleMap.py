class _UniffiHandleMap:
    """
    A map where inserting, getting and removing data is synchronized with a lock.
    """

    def __init__(self):
        # type Handle = int
        self._map = {}  # type: Dict[Handle, Any]
        self._lock = threading.Lock()
        # Start at 1, since `0` represents a NULL handle.
        self._counter = itertools.count(1)

    def insert(self, obj):
        with self._lock:
            handle = next(self._counter)
            self._map[handle] = obj
            return handle

    def get(self, handle):
        try:
            with self._lock:
                return self._map[handle]
        except KeyError:
            raise InternalError("UniffiHandleMap.get: Invalid handle")

    def remove(self, handle):
        try:
            with self._lock:
                return self._map.pop(handle)
        except KeyError:
            raise InternalError("UniffiHandleMap.remove: Invalid handle")

    def __len__(self):
        return len(self._map)
