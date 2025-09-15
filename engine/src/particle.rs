use std::mem;

use glam::Vec2;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub const MAX_INSTANCES: usize = 50_000;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub mass: f32,
    pub color: [f32; 3],
}

impl Particle {
    pub fn new(position: Vec2, velocity: Vec2, radius: f32, mass: f32, color: [f32; 3]) -> Self {
        Self {
            position,
            velocity,
            radius,
            mass,
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

    pub fn from_particle(p: &Particle) -> Self {
        Self {
            pos: [p.position.x, p.position.y],
            radius: p.radius,
            _pad0: 0.0,
            color: p.color,
            _pad1: 0.0,
        }
    }
}
