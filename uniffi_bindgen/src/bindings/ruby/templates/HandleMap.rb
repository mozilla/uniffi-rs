# A handle map for converting Ruby objects to/from integer handles.
#
# When passing callback interface or trait interface objects to Ruby, we store
# the Ruby object in this map and pass the integer handle to Rust. When Rust
# calls back, we look up the Ruby object by its handle.
#
# Ruby-created handles are always odd (staring at 1, incrementing by 2).
# Rust-created handles are always even. This convention lets us distinguish
# the source in lift/lower without extra metadata.


class UniffiHandleMap
  def initialize
    @lock = Monitor.new
    @map = {}
    @counter = 1 # Start at 1 (odd), increment by 2
  end

  def insert(obj)
    @lock.synchronize do
      handle = @counter
      @counter += 2
      @map[handle] = obj
      handle
    end
  end

  def get(handle)
    @lock.synchronize do
      obj = @map[handle]
      raise InternalError, "UniffiHandleMap.get: invalid handle #{handle}" if obj.nil?
      obj
    end
  end

  def clone_handle(handle)
    @lock.synchronize do
      obj = @map[handle]
      raise InternalError, "UniffiHandleMap.clone_handle: invalid handle #{handle}" if obj.nil?
      new_handle = @counter
      @counter += 2
      @map[new_handle] = obj
      new_handle
    end
  end

  def remove(handle)
    @lock.synchronize do
      obj = @map.delete(handle)
      raise InternalError, "UniffiHandleMap.remove: invalid handle #{handle}" if obj.nil?
      obj
    end
  end

  # Used for testing/debugging purposes
  def size
    @lock.synchronize { @map.size }
  end
end

private_constant :UniffiHandleMap
