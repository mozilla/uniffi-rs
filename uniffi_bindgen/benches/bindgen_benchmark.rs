/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use askama::Template;
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;
use uniffi_bindgen::{bindings::kotlin::KotlinWrapper, interface::ComponentInterface};

pub fn criterion_benchmark(c: &mut Criterion) {
    let udl = std::fs::read_to_string(
        "/Users/rfk/repos/mozilla/application-services/components/fxa-client/src/fxa_client.udl",
    )
    .unwrap();
    c.bench_function("bindgen fxa-client", |b| {
        b.iter(|| {
            let ci = udl.parse::<ComponentInterface>().unwrap();
            let bgen = KotlinWrapper::new(Default::default(), &ci);
            bgen.render().unwrap()
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = criterion_benchmark
}
criterion_main!(benches);
