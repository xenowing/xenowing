use crate::vec4::*;

use std::ops::Mul;

const NUM_ROWS: usize = 4;
const NUM_COLS: usize = 4;
const NUM_VALUES: usize = NUM_ROWS * NUM_COLS;

#[derive(Clone, Copy)]
pub struct Matrix {
    values: [f32; NUM_VALUES]
}

impl Matrix {
    pub fn from_floats(values: &[f32; NUM_VALUES]) -> Matrix {
        Matrix {
            values: values.clone(),
        }
    }

    pub fn from_doubles(values: &[f64; NUM_VALUES]) -> Matrix {
        let mut ret = Matrix {
            values: [0.0; NUM_VALUES],
        };
        for (i, v) in values.iter().enumerate() {
            ret.values[i] = *v as f32;
        }
        ret
    }

    pub fn identity() -> Matrix {
        Matrix {
            values: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Matrix {
        Matrix {
            values: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                x, y, z, 1.0]
        }
    }

    pub fn rotation_x(radians: f32) -> Matrix {
        let s = radians.sin();
        let c = radians.cos();

        Matrix {
            values: [
                1.0, 0.0, 0.0, 0.0,
                0.0, c, s, 0.0,
                0.0, -s, c, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn rotation_y(radians: f32) -> Matrix {
        let s = radians.sin();
        let c = radians.cos();

        Matrix {
            values: [
                c, 0.0, -s, 0.0,
                0.0, 1.0, 0.0, 0.0,
                s, 0.0, c, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn rotation_z(radians: f32) -> Matrix {
        let s = radians.sin();
        let c = radians.cos();

        Matrix {
            values: [
                c, s, 0.0, 0.0,
                -s, c, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> Matrix {
        let tx = -(right + left) / (right - left);
        let ty = -(top + bottom) / (top - bottom);
        let tz = -(z_far + z_near) / (z_far - z_near);

        Matrix {
            values: [
                2.0 / (right - left), 0.0, 0.0, 0.0,
                0.0, 2.0 / (top - bottom), 0.0, 0.0,
                0.0, 0.0, -2.0 / (z_far - z_near), 0.0,
                tx, ty, tz, 1.0]
        }
    }

    pub fn perspective(fov_degrees: f32, aspect: f32, z_near: f32, z_far: f32) -> Matrix {
        let fov_radians = fov_degrees.to_radians();
        let top = z_near * (fov_radians / 2.0).tan();
        let right = top * aspect;

        let z_range = z_far - z_near;

        Matrix {
            values: [
                z_near / right, 0.0, 0.0, 0.0,
                0.0, z_near / top, 0.0, 0.0,
                0.0, 0.0, -(z_near + z_far) / z_range, -1.0,
                0.0, 0.0, -2.0 * z_near * z_far / z_range, 0.0]
        }
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        &self * &other
    }
}

impl<'a> Mul<&'a Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, other: &'a Matrix) -> Matrix {
        &self * other
    }
}

impl<'a> Mul<Matrix> for &'a Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        self * &other
    }
}

impl<'a, 'b> Mul<&'a Matrix> for &'b Matrix {
    type Output = Matrix;

    fn mul(self, other: &'a Matrix) -> Matrix {
        Matrix {
            values: [
                (self.values[00] * other.values[00]) + (self.values[04] * other.values[01]) + (self.values[08] * other.values[02]) + (self.values[12] * other.values[03]),
                (self.values[01] * other.values[00]) + (self.values[05] * other.values[01]) + (self.values[09] * other.values[02]) + (self.values[13] * other.values[03]),
                (self.values[02] * other.values[00]) + (self.values[06] * other.values[01]) + (self.values[10] * other.values[02]) + (self.values[14] * other.values[03]),
                (self.values[03] * other.values[00]) + (self.values[07] * other.values[01]) + (self.values[11] * other.values[02]) + (self.values[15] * other.values[03]),
                (self.values[00] * other.values[04]) + (self.values[04] * other.values[05]) + (self.values[08] * other.values[06]) + (self.values[12] * other.values[07]),
                (self.values[01] * other.values[04]) + (self.values[05] * other.values[05]) + (self.values[09] * other.values[06]) + (self.values[13] * other.values[07]),
                (self.values[02] * other.values[04]) + (self.values[06] * other.values[05]) + (self.values[10] * other.values[06]) + (self.values[14] * other.values[07]),
                (self.values[03] * other.values[04]) + (self.values[07] * other.values[05]) + (self.values[11] * other.values[06]) + (self.values[15] * other.values[07]),
                (self.values[00] * other.values[08]) + (self.values[04] * other.values[09]) + (self.values[08] * other.values[10]) + (self.values[12] * other.values[11]),
                (self.values[01] * other.values[08]) + (self.values[05] * other.values[09]) + (self.values[09] * other.values[10]) + (self.values[13] * other.values[11]),
                (self.values[02] * other.values[08]) + (self.values[06] * other.values[09]) + (self.values[10] * other.values[10]) + (self.values[14] * other.values[11]),
                (self.values[03] * other.values[08]) + (self.values[07] * other.values[09]) + (self.values[11] * other.values[10]) + (self.values[15] * other.values[11]),
                (self.values[00] * other.values[12]) + (self.values[04] * other.values[13]) + (self.values[08] * other.values[14]) + (self.values[12] * other.values[15]),
                (self.values[01] * other.values[12]) + (self.values[05] * other.values[13]) + (self.values[09] * other.values[14]) + (self.values[13] * other.values[15]),
                (self.values[02] * other.values[12]) + (self.values[06] * other.values[13]) + (self.values[10] * other.values[14]) + (self.values[14] * other.values[15]),
                (self.values[03] * other.values[12]) + (self.values[07] * other.values[13]) + (self.values[11] * other.values[14]) + (self.values[15] * other.values[15])]
        }
    }
}

impl Mul<Vec4> for Matrix {
    type Output = Vec4;

    fn mul(self, other: Vec4) -> Vec4 {
        // TODO: Simplify as dot products with row vectors
        Vec4::new(
            self.values[00] * other.x() + self.values[04] * other.y() + self.values[08] * other.z() + self.values[12] * other.w(),
            self.values[01] * other.x() + self.values[05] * other.y() + self.values[09] * other.z() + self.values[13] * other.w(),
            self.values[02] * other.x() + self.values[06] * other.y() + self.values[10] * other.z() + self.values[14] * other.w(),
            self.values[03] * other.x() + self.values[07] * other.y() + self.values[11] * other.z() + self.values[15] * other.w()
        )
    }
}
