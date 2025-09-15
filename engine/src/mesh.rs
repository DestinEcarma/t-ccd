use std::mem;

use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
}

impl QuadVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<QuadVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x2,
            }],
        }
    }
}

pub const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex {
        position: [-1.0, -1.0],
    },
    QuadVertex {
        position: [1.0, -1.0],
    },
    QuadVertex {
        position: [1.0, 1.0],
    },
    QuadVertex {
        position: [-1.0, 1.0],
    },
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
