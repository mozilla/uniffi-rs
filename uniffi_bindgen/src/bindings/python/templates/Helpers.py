# Miscellaneous "private" helpers.
#
# These are things that we need to have available for internal use,
# but that we don't want to expose as part of the public API classes,
# even as "hidden" methods.

{% if ci.iter_object_definitions().len() > 0 %}
def liftObject(cls, handle):
    """Helper to create an object instace from a handle.

    This bypasses the usual __init__() logic for the given class
    and just directly creates an instance with the given handle.
    It's used to support factory functions and methods, which need
    to return instances without invoking a (Python-level) constructor.
    """
    obj = cls.__new__(cls)
    obj._handle = handle
    return obj
{% endif %}