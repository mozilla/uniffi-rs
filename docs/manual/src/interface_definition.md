# The Interface Definition

The public interface of your Rust crate should be defined as an inline module
decorated with the `#[uniffi::declare_interface]` macro. Code inside
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

// But its public interface should be defined inside a submodule,
// using the `declare_interface` macro.
#[uniffi::declare_interface]
mod sprites {

  // The interface can define data structs with public fields and no method impls.
  // We call these "records", and they are used to communicate structured data back-and-forth
  // with foreign-language code.

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

  // The interface can define opaque structs with method impls but no public fields.
  // We call these "objects" and they encapsulate behaviour and state, typically
  // mapping to the foreign-language equivalent of a "class" in the object-oriented sense.

  pub struct Sprite {
    // Object fields are private, since the foreign language can't access them directly.
    current_position: RwLock<Point>,
  }

  impl Sprite {
    // Each object should have a constructor named "new", which gets connected to
    // object initialization syntax in the foreign-language bindings.
    pub fn new(initial_position: Option<Point>) -> Sprite {
      Sprite {
        current_position: RwLock::new(initial_position.unwrap_or_default())
      }
    }

    // Public methods implemented on the struct will be callable
    // from the foreign-language bindings.

    // The internal state of the object can only be inspected by calling its public methods.
    pub fn get_position(&self) -> Point {
      self.current_position.read().unwrap().clone()
    }

    // Methods must use interior mutability rather than `&mut self` to modify internal state.
    pub fn move_to(&mut self, position: Point) {
      *self.current_position.write().unwrap() = position;
    }
  }
}
```
