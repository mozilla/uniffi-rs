/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// A point in two-dimensional space.
#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

// A magnitude and direction in two-dimensional space.
// For simplicity we represent this as a point relative to the origin.
#[derive(Debug, Clone)]
struct Vector {
    dx: f64,
    dy: f64,
}

// Move from the given Point, according to the given Vector.
fn translate(p: Point, v: Vector) -> Point {
    Point {
        x: p.x + v.dx,
        y: p.y + v.dy,
    }
}

// An entity in our imaginary world, which occupies a position in space
// and which can move about over time.
#[derive(Debug, Clone)]
struct Sprite {
    current_position: Point,
}

impl Sprite {
    fn new(initial_position: Option<Point>) -> Sprite {
        Sprite {
            current_position: initial_position.unwrap_or_else(|| Point { x: 0.0, y: 0.0 }),
        }
    }

    fn get_position(&self) -> Point {
        self.current_position.clone()
    }

    fn move_to(&mut self, position: Point) {
        self.current_position = position;
    }

    fn move_by(&mut self, direction: Vector) {
        self.current_position = translate(self.current_position.clone(), direction)
    }
}

include!(concat!(env!("OUT_DIR"), "/sprites.uniffi.rs"));
