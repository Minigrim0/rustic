#[derive(Debug)]
pub struct Text {
    pub position: cgmath::Vector2<f32>,
    pub bounds: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>,
    pub text: String,
    pub size: f32,
    pub visible: bool,
    pub focused: bool,
    pub centered: bool,
}
