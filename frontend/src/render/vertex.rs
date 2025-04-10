#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: cgmath::Vector2<f32>,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: Self::SIZE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
        ],
    };

    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;

    pub fn new(position: (f32, f32)) -> Self {
        Self {
            position: position.into(),
        }
    }
}
