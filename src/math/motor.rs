use crate::math::Vector3;
use encase::ShaderType;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Motor {
    pub s: f32,
    pub e12: f32,
    pub e13: f32,
    pub e23: f32,
    pub e01: f32,
    pub e02: f32,
    pub e03: f32,
    pub e0123: f32,
}

impl Motor {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
        e0123: 0.0,
    };

    pub fn translation(offset: Vector3) -> Self {
        Self {
            s: 1.0,
            e12: 0.0,
            e13: 0.0,
            e23: 0.0,
            e01: offset.x * -0.5,
            e02: offset.y * -0.5,
            e03: offset.z * -0.5,
            e0123: 0.0,
        }
    }

    pub fn rotation_xy(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e12: sin,
            e13: 0.0,
            e23: 0.0,
            e01: 0.0,
            e02: 0.0,
            e03: 0.0,
            e0123: 0.0,
        }
    }

    pub fn rotation_xz(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e12: 0.0,
            e13: sin,
            e23: 0.0,
            e01: 0.0,
            e02: 0.0,
            e03: 0.0,
            e0123: 0.0,
        }
    }

    pub fn rotation_yz(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e12: 0.0,
            e13: 0.0,
            e23: sin,
            e01: 0.0,
            e02: 0.0,
            e03: 0.0,
            e0123: 0.0,
        }
    }

    pub fn apply(self, other: Self) -> Self {
        let a = self.s;
        let b = self.e12;
        let c = self.e13;
        let d = self.e23;
        let e = self.e01;
        let f = self.e02;
        let g = self.e03;
        let h = self.e0123;
        let i = other.s;
        let j = other.e12;
        let k = other.e13;
        let l = other.e23;
        let m = other.e01;
        let n = other.e02;
        let o = other.e03;
        let p = other.e0123;

        /*
        Combining Motors

        (a + b*e1*e2 + c*e1*e3 + d*e2*e3 + e*e0*e1 + f*e0*e2 + g*e0*e3 + h*e0*e1*e2*e3)
        *(i + j*e1*e2 + k*e1*e3 + l*e2*e3 + m*e0*e1 + n*e0*e2 + o*e0*e3 + p*e0*e1*e2*e3)

        -1*b*j + -1*c*k + -1*d*l + a*i
        + (-1*c*l + a*j + b*i + d*k)*e1*e2
        + (-1*d*j + a*k + b*l + c*i)*e1*e3
        + (-1*b*k + a*l + c*j + d*i)*e2*e3
        + (-1*d*p + -1*f*j + -1*g*k + -1*h*l + a*m + b*n + c*o + e*i)*e0*e1
        + (-1*b*m + -1*g*l + a*n + c*p + d*o + e*j + f*i + h*k)*e0*e2
        + (-1*b*p + -1*c*m + -1*d*n + -1*h*j + a*o + e*k + f*l + g*i)*e0*e3
        + (-1*c*n + -1*f*k + a*p + b*o + d*m + e*l + g*j + h*i)*e0*e1*e2*e3
        */

        Self {
            s: -b * j + -c * k + -d * l + a * i,
            e12: -c * l + a * j + b * i + d * k,
            e13: -d * j + a * k + b * l + c * i,
            e23: -b * k + a * l + c * j + d * i,
            e01: -d * p + -f * j + -g * k + -h * l + a * m + b * n + c * o + e * i,
            e02: -b * m + -g * l + a * n + c * p + d * o + e * j + f * i + h * k,
            e03: -b * p + -c * m + -d * n + -h * j + a * o + e * k + f * l + g * i,
            e0123: -c * n + -f * k + a * p + b * o + d * m + e * l + g * j + h * i,
        }
    }

    pub fn pre_apply(self, other: Self) -> Self {
        other.apply(self)
    }

    pub fn inverse(self) -> Self {
        Self {
            s: self.s,
            e12: -self.e12,
            e13: -self.e13,
            e23: -self.e23,
            e01: -self.e01,
            e02: -self.e02,
            e03: -self.e03,
            e0123: self.e0123,
        }
    }

    pub fn rotation_part(self) -> Self {
        Self {
            s: self.s,
            e12: self.e12,
            e13: self.e13,
            e23: self.e23,
            e01: 0.0,
            e02: 0.0,
            e03: 0.0,
            e0123: 0.0,
        }
    }
}
