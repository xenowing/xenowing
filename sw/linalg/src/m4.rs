use crate::v4::*;

use core::intrinsics;
use core::ops::Mul;

const NUM_ROWS: usize = 4;
const NUM_COLS: usize = 4;
const NUM_VALUES: usize = NUM_ROWS * NUM_COLS;

#[derive(Clone, Copy)]
pub struct M4 {
    values: [f32; NUM_VALUES]
}

impl M4 {
    pub fn from_floats(values: &[f32; NUM_VALUES]) -> M4 {
        M4 {
            values: values.clone(),
        }
    }

    pub fn from_doubles(values: &[f64; NUM_VALUES]) -> M4 {
        let mut ret = M4 {
            values: [0.0; NUM_VALUES],
        };
        for (i, v) in values.iter().enumerate() {
            ret.values[i] = *v as f32;
        }
        ret
    }

    pub fn identity() -> M4 {
        M4 {
            values: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn translation(x: f32, y: f32, z: f32) -> M4 {
        M4 {
            values: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                x, y, z, 1.0]
        }
    }

    pub fn rotation_x(radians: f32) -> M4 {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        M4 {
            values: [
                1.0, 0.0, 0.0, 0.0,
                0.0, c, s, 0.0,
                0.0, -s, c, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn rotation_y(radians: f32) -> M4 {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        M4 {
            values: [
                c, 0.0, -s, 0.0,
                0.0, 1.0, 0.0, 0.0,
                s, 0.0, c, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn rotation_z(radians: f32) -> M4 {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        M4 {
            values: [
                c, s, 0.0, 0.0,
                -s, c, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn scale(x: f32, y: f32, z: f32) -> M4 {
        M4 {
            values: [
                x, 0.0, 0.0, 0.0,
                0.0, y, 0.0, 0.0,
                0.0, 0.0, z, 0.0,
                0.0, 0.0, 0.0, 1.0]
        }
    }

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> M4 {
        let tx = -(right + left) / (right - left);
        let ty = -(top + bottom) / (top - bottom);
        let tz = -(z_far + z_near) / (z_far - z_near);

        M4 {
            values: [
                2.0 / (right - left), 0.0, 0.0, 0.0,
                0.0, 2.0 / (top - bottom), 0.0, 0.0,
                0.0, 0.0, -2.0 / (z_far - z_near), 0.0,
                tx, ty, tz, 1.0]
        }
    }

    pub fn perspective(fov_degrees: f32, aspect: f32, z_near: f32, z_far: f32) -> M4 {
        let fov_radians = fov_degrees.to_radians();
        let tan = |x| unsafe { intrinsics::sinf32(x) / intrinsics::cosf32(x) };
        let top = z_near * tan(fov_radians / 2.0);
        let right = top * aspect;

        let z_range = z_far - z_near;

        M4 {
            values: [
                z_near / right, 0.0, 0.0, 0.0,
                0.0, z_near / top, 0.0, 0.0,
                0.0, 0.0, -(z_near + z_far) / z_range, -1.0,
                0.0, 0.0, -2.0 * z_near * z_far / z_range, 0.0]
        }
    }
}

impl Mul<M4> for M4 {
    type Output = M4;

    fn mul(self, other: M4) -> M4 {
        &self * &other
    }
}

impl<'a> Mul<&'a M4> for M4 {
    type Output = M4;

    fn mul(self, other: &'a M4) -> M4 {
        &self * other
    }
}

impl<'a> Mul<M4> for &'a M4 {
    type Output = M4;

    fn mul(self, other: M4) -> M4 {
        self * &other
    }
}

impl<'a, 'b> Mul<&'a M4> for &'b M4 {
    type Output = M4;

    fn mul(self, other: &'a M4) -> M4 {
        // TODO: Simplify as dot products with row/column vectors
        M4 {
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

impl Mul<V4> for M4 {
    type Output = V4;

    fn mul(self, other: V4) -> V4 {
        // TODO: Simplify as dot products with row vectors
        V4::new(
            self.values[00] * other.x() + self.values[04] * other.y() + self.values[08] * other.z() + self.values[12] * other.w(),
            self.values[01] * other.x() + self.values[05] * other.y() + self.values[09] * other.z() + self.values[13] * other.w(),
            self.values[02] * other.x() + self.values[06] * other.y() + self.values[10] * other.z() + self.values[14] * other.w(),
            self.values[03] * other.x() + self.values[07] * other.y() + self.values[11] * other.z() + self.values[15] * other.w()
        )
    }
}
