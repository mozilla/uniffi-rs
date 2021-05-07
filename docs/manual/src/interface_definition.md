# The Interface Definition

The public interface of your Rust crate should be defined as an inline module
decorated with the `#[uniffi_macros::declare_interface]` macro. Code inside
this macro is restricted to a small subset of the Rust language that UniFFI
understands and that can be safely exposed to other languages.

Here is a small but relatively complete example:

```rs
//!
//! This is an example Rust crate to be exposed via UniFFI.
//! It's a subset of our "sprites" example crate.
//!

// It can use things from stdlib, submodules, or other Rust crates.
use std::io::prelude::*;

// But its public interface should be defined inside the `declare_interface` macro.
#[uniffi_macros::declare_interface]
mod sprites {

  // The interface can define "records" as structs with public fields.
  // These are used to communicate structured data back-and-forth with foreign-language code.
  // They do not have any public method impls, they're just containers for structured data
  // (but they can use some standard Rust derive macros).

  #[derive(Default)]
  pub struct Point {
    pub x: f64,
    pub y: f64,
  };

  pub struct Vector {
    pub dx: f64,
    pub dy: f64,
  };

  // The interface can expose functions that are callable for foreign-language code.

  pub fn translate(position: &Point, direction: &Vector) -> Point {
    // Ordinary Rust code goes here, perhaps calling out to sub-modules
    // that contain the bulk of the implementation.
    Point {
        x: p.x + v.dx,
        y: p.y + v.dy,
    }
  }

  // The interface can define "objects" as structs with private fields and
  // method implementations. These are used to represent objects with encapsulated
  // behaviour and state, so they can have public methods but no public fields.

  pub struct Sprite {
    // Object fields are private, since the foreign language can't access them directly.
    current_position: Point,
  }

  impl Sprite {
    // Each object should have a constructor named "new", which gets connected to
    // object initialization syntax in the foreign-language bindings.
    pub fn new(initial_position: Option<Point>) -> Sprite {
      Sprite {
        current_position: initial_position.unwrap_or_default()
      }
    }

    // Public methods implemented on the struct will be callable from the foreign-language
    // bindings. UniFFI guarantees that the usual Rust invariants around `&mut` and safety
    // will be upheld at runtime.
    pub fn move_to(&mut self, position: Point) {
      self.current_position = position
    }

    pub fn move_by(&mut self, direction: &Vector) {
      self.current_position = translate(&self.current_position, direction)
    }
  }
}
```
