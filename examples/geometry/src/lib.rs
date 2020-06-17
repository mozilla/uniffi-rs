/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone)]
struct Line {
    start: Point,
    end: Point,
}

fn gradient(ln: Line) -> f64 {
    let rise = ln.end.y - ln.start.y;
    let run = ln.end.x - ln.start.x;
    rise / run
}

fn intersection(ln1: Line, ln2: Line) -> Option<Point> {
    // TODO: yuck, should be able to take &Line as argument here
    // and have rust figure it out with a bunch of annotations...
    let g1 = gradient(ln1.clone());
    let z1 = ln1.start.y - g1 * ln1.start.x;
    let g2 = gradient(ln2.clone());
    let z2 = ln2.start.y - g2 * ln2.start.x;
    // Parallel lines do not intersect.
    if g1 == g2 {
        return None;
    }
    // Otherwise, they intersect at this fancy calculation that
    // I found on wikipedia.
    let x = (z2 - z1) / (g1 - g2);
    Some(Point {
        x: x,
        y: g1 * x + z1,
    })
}

include!(concat!(env!("OUT_DIR"), "/geometry.uniffi.rs"));
