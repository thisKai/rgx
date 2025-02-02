use std::f32;

use crate::math::*;

use crate::core;
use crate::core::{Binding, BindingType, Rgba, Set, ShaderStage};
use crate::rect::Rect;

use crate::kit::{Model, Rgba8, ZDepth};

///////////////////////////////////////////////////////////////////////////
// Uniforms
///////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Uniforms {
    pub ortho: Matrix4<f32>,
    pub transform: Matrix4<f32>,
}

///////////////////////////////////////////////////////////////////////////
// Vertex
///////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: Vector3<f32>,
    angle: f32,
    center: Vector2<f32>,
    color: Rgba8,
}

impl Vertex {
    const fn new(x: f32, y: f32, z: f32, angle: f32, center: Point2<f32>, color: Rgba8) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            angle,
            center: Vector2::new(center.x, center.y),
            color,
        }
    }
}

#[inline]
pub const fn vertex(
    x: f32,
    y: f32,
    z: f32,
    angle: f32,
    center: Point2<f32>,
    color: Rgba8,
) -> Vertex {
    Vertex::new(x, y, z, angle, center, color)
}

///////////////////////////////////////////////////////////////////////////
// Pipeline
///////////////////////////////////////////////////////////////////////////

pub struct Pipeline {
    pipeline: core::Pipeline,
    bindings: core::BindingGroup,
    buf: core::UniformBuffer,
    model: Model,
}

//////////////////////////////////////////////////////////////////////////

impl<'a> core::AbstractPipeline<'a> for Pipeline {
    type PrepareContext = Matrix4<f32>;
    type Uniforms = self::Uniforms;

    fn description() -> core::PipelineDescription<'a> {
        core::PipelineDescription {
            vertex_layout: &[
                // Position
                core::VertexFormat::Float3,
                // Roation angle.
                core::VertexFormat::Float,
                // Center of rotation.
                core::VertexFormat::Float2,
                // Color
                core::VertexFormat::UByte4,
            ],
            pipeline_layout: &[
                Set(&[Binding {
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
                Set(&[Binding {
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
            ],
            // TODO: Use `env("CARGO_MANIFEST_DIR")`
            vertex_shader: include_bytes!("data/shape.vert.spv"),
            fragment_shader: include_bytes!("data/shape.frag.spv"),
        }
    }

    fn setup(pipeline: core::Pipeline, dev: &core::Device) -> Self {
        let transform = Matrix4::identity();
        let ortho = Matrix4::identity();
        let model = Model::new(&pipeline.layout.sets[1], &[Matrix4::identity()], dev);
        let buf = dev.create_uniform_buffer(&[self::Uniforms { ortho, transform }]);
        let bindings = dev.create_binding_group(&pipeline.layout.sets[0], &[&buf]);

        Self {
            pipeline,
            buf,
            bindings,
            model,
        }
    }

    fn apply(&self, pass: &mut core::Pass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_binding(&self.bindings, &[]);
        pass.set_binding(&self.model.binding, &[]);
    }

    fn prepare(
        &'a self,
        ortho: Matrix4<f32>,
    ) -> Option<(&'a core::UniformBuffer, Vec<self::Uniforms>)> {
        let transform = Matrix4::identity();
        Some((&self.buf, vec![self::Uniforms { transform, ortho }]))
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Shapes
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Stroke {
    width: f32,
    color: Rgba,
}

impl Stroke {
    pub const NONE: Self = Self {
        width: 0.,
        color: Rgba::TRANSPARENT,
    };

    pub fn new(width: f32, color: Rgba) -> Self {
        Self { width, color }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Fill {
    Empty(),
    Solid(Rgba),
    Gradient(Rgba, Rgba),
}

#[derive(Clone, Debug)]
pub struct Rotation {
    angle: f32,
    center: Point2<f32>,
}

impl Rotation {
    pub const ZERO: Rotation = Rotation {
        angle: 0.0,
        center: Point2 { x: 0.0, y: 0.0 },
    };

    pub fn new(angle: f32, center: Point2<f32>) -> Self {
        Self { angle, center }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::ZERO
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    Line(Line, ZDepth, Rotation, Stroke),
    Rectangle(Rect<f32>, ZDepth, Rotation, Stroke, Fill),
    Circle(Point2<f32>, ZDepth, f32, u32, Stroke, Fill),
}

impl Shape {
    pub fn triangulate(&self) -> Vec<Vertex> {
        match *self {
            Shape::Line(l, ZDepth(z), Rotation { angle, center }, Stroke { width, color }) => {
                let v = (l.p2 - l.p1).normalize();

                let wx = width / 2.0 * v.y;
                let wy = width / 2.0 * v.x;
                let rgba8 = color.into();

                vec![
                    vertex(l.p1.x - wx, l.p1.y + wy, z, angle, center, rgba8),
                    vertex(l.p1.x + wx, l.p1.y - wy, z, angle, center, rgba8),
                    vertex(l.p2.x - wx, l.p2.y + wy, z, angle, center, rgba8),
                    vertex(l.p2.x - wx, l.p2.y + wy, z, angle, center, rgba8),
                    vertex(l.p1.x + wx, l.p1.y - wy, z, angle, center, rgba8),
                    vertex(l.p2.x + wx, l.p2.y - wy, z, angle, center, rgba8),
                ]
            }
            Shape::Rectangle(r, ZDepth(z), Rotation { angle, center }, stroke, fill) => {
                let width = stroke.width;
                let inner = Rect::new(r.x1 + width, r.y1 + width, r.x2 - width, r.y2 - width);

                let mut verts = if stroke != Stroke::NONE {
                    let rgba8 = stroke.color.into();

                    let outer = r;

                    vec![
                        // Bottom
                        vertex(outer.x1, outer.y1, z, angle, center, rgba8),
                        vertex(outer.x2, outer.y1, z, angle, center, rgba8),
                        vertex(inner.x1, inner.y1, z, angle, center, rgba8),
                        vertex(inner.x1, inner.y1, z, angle, center, rgba8),
                        vertex(outer.x2, outer.y1, z, angle, center, rgba8),
                        vertex(inner.x2, inner.y1, z, angle, center, rgba8),
                        // Left
                        vertex(outer.x1, outer.y1, z, angle, center, rgba8),
                        vertex(inner.x1, inner.y1, z, angle, center, rgba8),
                        vertex(outer.x1, outer.y2, z, angle, center, rgba8),
                        vertex(outer.x1, outer.y2, z, angle, center, rgba8),
                        vertex(inner.x1, inner.y1, z, angle, center, rgba8),
                        vertex(inner.x1, inner.y2, z, angle, center, rgba8),
                        // Right
                        vertex(inner.x2, inner.y1, z, angle, center, rgba8),
                        vertex(outer.x2, outer.y1, z, angle, center, rgba8),
                        vertex(outer.x2, outer.y2, z, angle, center, rgba8),
                        vertex(inner.x2, inner.y1, z, angle, center, rgba8),
                        vertex(inner.x2, inner.y2, z, angle, center, rgba8),
                        vertex(outer.x2, outer.y2, z, angle, center, rgba8),
                        // Top
                        vertex(outer.x1, outer.y2, z, angle, center, rgba8),
                        vertex(outer.x2, outer.y2, z, angle, center, rgba8),
                        vertex(inner.x1, inner.y2, z, angle, center, rgba8),
                        vertex(inner.x1, inner.y2, z, angle, center, rgba8),
                        vertex(outer.x2, outer.y2, z, angle, center, rgba8),
                        vertex(inner.x2, inner.y2, z, angle, center, rgba8),
                    ]
                } else {
                    Vec::with_capacity(6)
                };

                match fill {
                    Fill::Solid(color) => {
                        let rgba8 = color.into();

                        verts.extend_from_slice(&[
                            vertex(inner.x1, inner.y1, z, angle, center, rgba8),
                            vertex(inner.x2, inner.y1, z, angle, center, rgba8),
                            vertex(inner.x2, inner.y2, z, angle, center, rgba8),
                            vertex(inner.x1, inner.y1, z, angle, center, rgba8),
                            vertex(inner.x1, inner.y2, z, angle, center, rgba8),
                            vertex(inner.x2, inner.y2, z, angle, center, rgba8),
                        ]);
                    }
                    Fill::Gradient(_, _) => {
                        unimplemented!();
                    }
                    Fill::Empty() => {}
                }
                verts
            }
            Shape::Circle(position, ZDepth(z), radius, sides, stroke, fill) => {
                let inner = Self::circle(position, radius - stroke.width, sides);

                let mut verts = if stroke != Stroke::NONE {
                    // If there is a stroke, the outer circle is larger.
                    let outer = Self::circle(position, radius, sides);
                    let rgba8 = stroke.color.into();

                    let n = inner.len() - 1;
                    let mut vs = Vec::with_capacity(n * 6);
                    for i in 0..n {
                        let (i0, i1) = (inner[i], inner[i + 1]);
                        let (o0, o1) = (outer[i], outer[i + 1]);

                        vs.extend_from_slice(&[
                            vertex(i0.x, i0.y, z, 0.0, Point2::new(0.0, 0.0), rgba8),
                            vertex(o0.x, o0.y, z, 0.0, Point2::new(0.0, 0.0), rgba8),
                            vertex(o1.x, o1.y, z, 0.0, Point2::new(0.0, 0.0), rgba8),
                            vertex(i0.x, i0.y, z, 0.0, Point2::new(0.0, 0.0), rgba8),
                            vertex(o1.x, o1.y, z, 0.0, Point2::new(0.0, 0.0), rgba8),
                            vertex(i1.x, i1.y, z, 0.0, Point2::new(0.0, 0.0), rgba8),
                        ]);
                    }
                    vs
                } else {
                    Vec::new()
                };

                match fill {
                    Fill::Solid(color) => {
                        let rgba8 = color.into();
                        let center = Vertex::new(
                            position.x,
                            position.y,
                            z,
                            0.0,
                            Point2::new(0.0, 0.0),
                            rgba8,
                        );
                        let inner_verts: Vec<Vertex> = inner
                            .iter()
                            .map(|p| Vertex::new(p.x, p.y, z, 0., Point2::new(0.0, 0.0), rgba8))
                            .collect();
                        for i in 0..sides as usize {
                            verts.extend_from_slice(&[center, inner_verts[i], inner_verts[i + 1]]);
                        }
                        verts.extend_from_slice(&[
                            center,
                            *inner_verts.last().unwrap(),
                            *inner_verts.first().unwrap(),
                        ]);
                    }
                    Fill::Gradient(_, _) => {
                        unimplemented!();
                    }
                    Fill::Empty() => {}
                }
                verts
            }
        }
    }

    fn circle(position: Point2<f32>, radius: f32, sides: u32) -> Vec<Point2<f32>> {
        let mut verts = Vec::with_capacity(sides as usize + 1);

        for i in 0..=sides as usize {
            let angle: f32 = i as f32 * ((2. * f32::consts::PI) / sides as f32);
            verts.push(Point2::new(
                position.x + radius * angle.cos(),
                position.y + radius * angle.sin(),
            ));
        }
        verts
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Line {
    pub p1: Vector2<f32>,
    pub p2: Vector2<f32>,
}

impl Line {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            p1: Vector2::new(x1, y1),
            p2: Vector2::new(x2, y2),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Batch
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Batch {
    items: Vec<Shape>,
}

impl Batch {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn singleton(shape: Shape) -> Self {
        let mut sv = Self::new();
        sv.add(shape);
        sv
    }

    pub fn add(&mut self, shape: Shape) {
        self.items.push(shape);
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        // TODO: This is a lower-bound estimate of how much space we need.
        // We should get the actual numbers from the shapes.
        let mut buf = Vec::with_capacity(6 * self.items.len());

        for shape in self.items.iter() {
            let mut verts: Vec<Vertex> = shape.triangulate();
            buf.append(&mut verts);
        }
        buf
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn buffer(&self, r: &core::Renderer) -> core::VertexBuffer {
        let buf = self.vertices();
        r.device.create_buffer(buf.as_slice())
    }

    pub fn finish(self, r: &core::Renderer) -> core::VertexBuffer {
        self.buffer(r)
    }
}
