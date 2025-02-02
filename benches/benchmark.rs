#[macro_use]
extern crate criterion;

use criterion::Criterion;

use rgx::math::Point2;

use rgx::core::{Rect, Rgba};
use rgx::kit::shape2d::*;

fn bench_triangulate_circle() {
    Shape::Circle(
        Point2::new(0., 0.),
        1.,
        64,
        Stroke::new(1., Rgba::WHITE),
        Fill::Solid(Rgba::WHITE),
    )
    .triangulate();
}

fn bench_triangulate_rectangle() {
    Shape::Rectangle(
        Rect::new(1., 1., 3., 3.),
        Rotation::ZERO,
        Stroke::new(1., Rgba::WHITE),
        Fill::Solid(Rgba::WHITE),
    )
    .triangulate();
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("triangulate circle", |b| {
        b.iter(|| bench_triangulate_circle())
    });
    c.bench_function("triangulate rectangle", |b| {
        b.iter(|| bench_triangulate_rectangle())
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
