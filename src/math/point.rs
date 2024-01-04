use crate::math::{Motor, Vector3};
use encase::ShaderType;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Point {
    pub e012: f32,
    pub e013: f32,
    pub e023: f32,
    pub e123: f32,
}

impl Point {
    pub const IDENTITY: Self = Self {
        e012: 0.0,
        e013: 0.0,
        e023: 0.0,
        e123: 1.0,
    };

    pub fn transform(self, motor: Motor) -> Self {
        let a = motor.s;
        let b = motor.e12;
        let c = motor.e13;
        let d = motor.e23;
        let e = motor.e01;
        let f = motor.e02;
        let g = motor.e03;
        let h = motor.e0123;
        let i = self.e012;
        let j = self.e013;
        let k = self.e023;
        let l = self.e123;

        /*
        Apply motor to point

        (a + b*e2*e1 + c*e3*e1 + d*e3*e2 + e*e1*e0 + f*e2*e0 + g*e3*e0 + h*e3*e2*e1*e0)
        *(i*e0*e1*e2 + j*e0*e1*e3 + k*e0*e2*e3 + l*e1*e2*e3)
        *(a + b*e1*e2 + c*e1*e3 + d*e2*e3 + e*e0*e1 + f*e0*e2 + g*e0*e3 + h*e0*e1*e2*e3)

        (
              -2*a*d*j + -2*a*g*l +   a*a*i + 2*a*c*k
            + -1*d*d*i + -2*d*f*l + 2*b*d*k + -2*b*h*l
            + -2*c*e*l +    b*b*i + 2*b*c*j + -1*c*c*i
        )*e0*e1*e2
        +
        (
              -2*a*b*k + -1*b*b*j + 2*b*c*i +  2*b*e*l
            +    a*a*j +  2*a*d*i + 2*a*f*l + -2*c*h*l
            + -2*d*g*l + -1*d*d*j + 2*c*d*k +    c*c*j
        )*e0*e1*e3
        +
        (
              -2*a*c*i + -2*a*e*l +   a*a*k +  2*a*b*j
            + -1*c*c*k +  2*c*d*j + 2*c*g*l + -2*d*h*l
            +  2*b*f*l + -1*b*b*k + 2*b*d*i +    d*d*k
        )*e0*e2*e3
        +
        (
            a*a*l + b*b*l + c*c*l + d*d*l
        )*e1*e2*e3

        */

        Self {
            e012: -2.0 * a * d * j
                + -2.0 * a * g * l
                + 1.0 * a * a * i
                + 2.0 * a * c * k
                + -1.0 * d * d * i
                + -2.0 * d * f * l
                + 2.0 * b * d * k
                + -2.0 * b * h * l
                + -2.0 * c * e * l
                + 1.0 * b * b * i
                + 2.0 * b * c * j
                + -1.0 * c * c * i,
            e013: -2.0 * a * b * k
                + -1.0 * b * b * j
                + 2.0 * b * c * i
                + 2.0 * b * e * l
                + 1.0 * a * a * j
                + 2.0 * a * d * i
                + 2.0 * a * f * l
                + -2.0 * c * h * l
                + -2.0 * d * g * l
                + -1.0 * d * d * j
                + 2.0 * c * d * k
                + 1.0 * c * c * j,
            e023: -2.0 * a * c * i
                + -2.0 * a * e * l
                + 1.0 * a * a * k
                + 2.0 * a * b * j
                + -1.0 * c * c * k
                + 2.0 * c * d * j
                + 2.0 * c * g * l
                + -2.0 * d * h * l
                + 2.0 * b * f * l
                + -1.0 * b * b * k
                + 2.0 * b * d * i
                + 1.0 * d * d * k,
            e123: a * a * l + b * b * l + c * c * l + d * d * l,
        }
    }
}

impl From<Vector3> for Point {
    fn from(value: Vector3) -> Self {
        Self {
            e012: value.z,
            e013: -value.y,
            e023: value.x,
            e123: 1.0,
        }
    }
}

impl From<Point> for Vector3 {
    fn from(value: Point) -> Self {
        Self {
            x: value.e023 / value.e123,
            y: -value.e013 / value.e123,
            z: value.e012 / value.e123,
        }
    }
}
