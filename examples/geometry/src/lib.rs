/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

include!(concat!(env!("OUT_DIR"), "/geometry.uniffi.rs"));

impl Geometry {
  fn gradient(ln: Line) -> u32 {
    let rise = ln.p2.y - ln.p1.y;
    let run = ln.p2.x - ln.p1.x;
    rise / run
  }

  fn intersection(ln1: Line, ln2: Line) -> Point {
    // XXX TODO: actually implement this.
    Point { x: 13, y: 42 }
  }
}
