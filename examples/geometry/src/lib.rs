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
    // TODO: yuck, should be able to take &Line as argument here
    // and have rust figure it out with a bunch of annotations...
    let g1 = Geometry::gradient(ln1.clone());
    let z1 = ln1.p1.y - g1 * ln1.p1.x;
    let g2 = Geometry::gradient(ln2.clone());
    let z2 = ln2.p1.y - g1 * ln2.p1.x;
    let i = (z2 - z1) / (g1 - g2);
    Point {
      x: i,
      y: g1 * i + z1,
    }
  }
}
