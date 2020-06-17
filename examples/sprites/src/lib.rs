/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

include!(concat!(env!("OUT_DIR"), "/sprites.uniffi.rs"));

fn translate(p: Point, v: Vector) -> Point {
  Point {x: p.x + v.dx, y: p.y + v.dy }
}

#[derive(Debug)]
struct Sprite {
  current_position: Point,
}

impl Sprite {
  fn new(initial_position: Point) -> Sprite {
    Sprite {
      current_position: initial_position,
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