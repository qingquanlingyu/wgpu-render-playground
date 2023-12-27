use cgmath::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

fn get_light_view_matrix(position: [f32; 3]) -> [[f32; 4]; 4]{
    let view_position:Point3<f32> = position.into();
    //let proj = cgmath::ortho(-80.0, 80.0, -80.0, 80.0, -200.0, 300.0);
    let projection = PerspectiveFov {
        fovy: Deg(90.0).into(),
        aspect: 1.0,
        near: 0.1,
        far: 100.0,
    };
    let view = Matrix4::look_at_rh(
        view_position,
        Point3::origin(),
        Vector3::unit_z(),
    );
    let view_proj = OPENGL_TO_WGPU_MATRIX * cgmath::Matrix4::from(projection.to_perspective()) * view;
    view_proj.into()
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    // 由于 Uniform 需要字段按 16 字节对齐，我们需要在这里使用一个填充字段
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
    proj: [[f32; 4]; 4],
}

impl LightUniform {
    pub fn new(position: [f32; 3], _padding:u32, color: [f32; 3], _padding2:u32) -> Self {
        let proj = get_light_view_matrix(position);
        Self {
            position,
            _padding,
            color,
            _padding2,
            proj,
        }
    }
    pub fn set_position (&mut self, position: [f32; 3]) {
        self.position = position;
        self.proj = get_light_view_matrix(position);
    }
    pub fn get_position (&self) -> [f32; 3] {
        self.position
    }
    pub fn get_proj (&self) -> [[f32; 4]; 4] {
        self.proj
    }
}