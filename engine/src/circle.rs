use std::mem;

use glam::Vec2;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub const MAX_INSTANCES: usize = 50_000;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Circle {
    pub position: Vec2,
    pub radius: f32,
    pub color: [f32; 3],
}

impl Circle {
    pub fn new(position: Vec2, radius: f32, color: [f32; 3]) -> Self {
        Self {
            position,
            radius,
            color,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub pos: [f32; 2],
    pub radius: f32,
    pub _pad0: f32,
    pub color: [f32; 3],
    pub _pad1: f32,
}

impl From<Circle> for InstanceRaw {
    fn from(c: Circle) -> Self {
        Self {
            pos: [c.position.x, c.position.y],
            radius: c.radius,
            _pad0: 0.0,
            color: Self::hsl_to_rgb(c.color[0], c.color[1], c.color[2]),
            _pad1: 0.0,
        }
    }
}

impl InstanceRaw {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as u64,
                    shader_location: 2,
                    format: VertexFormat::Float32,
                },
                VertexAttribute {
                    offset: (mem::size_of::<[f32; 2]>()
                        + mem::size_of::<f32>()
                        + mem::size_of::<f32>()) as u64,
                    shader_location: 3,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }

    #[inline]
    pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
        let a = s * l.min(1.0 - l);

        let f = |n: f32| {
            let k = (n + h * 12.0) % 12.0;
            l - a * (-1.0f32).max((k - 3.0).min(9.0 - k).min(1.0))
        };

        [f(0.0), f(8.0), f(4.0)]
    }
}
