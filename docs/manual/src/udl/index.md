# The UDL file

A UDL file allows you to define an interface externally from your Rust code.
It defines which functions, methods and types are exposed to the foreign-language bindings.

For example, here we describe a [namespace](../types/namespace.md) with 2 [records](../types/records.md) and an [interface](../types/interfaces.md)

```udl
namespace sprites {
  Point translate([ByRef] Point position, Vector direction);
};

dictionary Point {
  double x;
  double y;
};

dictionary Vector {
  double dx;
  double dy;
};

interface Sprite {
  constructor(Point? initial_position);
  Point get_position();
  void move_to(Point position);
  void move_by(Vector direction);
};
```
