#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn sqr_length(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.sqr_length().sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        Self {
            x: self.x / length,
            y: self.y / length,
        }
    }
}

impl AsRef<[f32; 2]> for Vector2 {
    fn as_ref(&self) -> &[f32; 2] {
        unsafe { std::mem::transmute(self) }
    }
}

impl AsMut<[f32; 2]> for Vector2 {
    fn as_mut(&mut self) -> &mut [f32; 2] {
        unsafe { std::mem::transmute(self) }
    }
}

impl From<[f32; 2]> for Vector2 {
    fn from([x, y]: [f32; 2]) -> Self {
        Self { x, y }
    }
}

encase::impl_vector!(2, Vector2, f32; using AsRef AsMut From);
