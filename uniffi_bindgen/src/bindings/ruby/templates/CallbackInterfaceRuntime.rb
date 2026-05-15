# Base class for callback interface FfiConverters.
# Stores Ruby callback objects in a handle map, and converts to/from integer handles.

class CallbackInterfaceFfiConverter
  attr_reader :handle_map

  def initialize
    @handle_map = UniffiHandleMap.new
  end

  def lift(handle)
    @handle_map.get handle
  end

  def lower(cb)
    @handle_map.insert cb
  end
end

private_constant :CallbackInterfaceFfiConverter
