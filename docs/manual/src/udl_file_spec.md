# The UDL file

This file defines which functions, methods and types are exposed to the foreign-language bindings.

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
