#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    // 由于 Uniform 需要字段按 16 字节对齐，我们需要在这里使用一个填充字段
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

impl LightUniform {
    pub fn new(position: [f32; 3], _padding:u32, color: [f32; 3], _padding2:u32) -> Self {
        Self {
            position,
            _padding,
            color,
            _padding2,
        }
    }
    pub fn set_position (&mut self, position: [f32; 3]) {
        self.position = position;
    }
    pub fn get_position (&self) -> [f32; 3] {
        self.position
    }
}