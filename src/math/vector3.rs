#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl AsRef<[f32; 3]> for Vector3 {
    fn as_ref(&self) -> &[f32; 3] {
        unsafe { std::mem::transmute(self) }
    }
}

impl AsMut<[f32; 3]> for Vector3 {
    fn as_mut(&mut self) -> &mut [f32; 3] {
        unsafe { std::mem::transmute(self) }
    }
}

impl From<[f32; 3]> for Vector3 {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self { x, y, z }
    }
}

encase::impl_vector!(3, Vector3, f32; using AsRef AsMut From);
