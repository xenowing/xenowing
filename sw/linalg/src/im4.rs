use crate::fixed::*;
use crate::iv4::*;

use core::intrinsics;
use core::ops::{Mul, MulAssign};

#[derive(Clone, Copy)]
pub struct Im4<const FRACT_BITS: u32> {
    pub columns: [Iv4<FRACT_BITS>; 4],
}

impl<const FRACT_BITS: u32> Im4<FRACT_BITS> {
    pub fn identity() -> Self {
        Self {
            columns: [
                Iv4::new(1.0, 0.0, 0.0, 0.0),
                Iv4::new(0.0, 1.0, 0.0, 0.0),
                Iv4::new(0.0, 0.0, 1.0, 0.0),
                Iv4::new(0.0, 0.0, 0.0, 1.0),
            ],
        }
    }

    pub fn translation(
        x: impl Into<Fixed<FRACT_BITS>>,
        y: impl Into<Fixed<FRACT_BITS>>,
        z: impl Into<Fixed<FRACT_BITS>>,
    ) -> Self {
        Self {
            columns: [
                Iv4::new(1.0, 0.0, 0.0, 0.0),
                Iv4::new(0.0, 1.0, 0.0, 0.0),
                Iv4::new(0.0, 0.0, 1.0, 0.0),
                Iv4::new(x, y, z, 1.0),
            ],
        }
    }

    pub fn rotation_x(radians: f32) -> Self {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        Self {
            columns: [
                Iv4::new(1.0, 0.0, 0.0, 0.0),
                Iv4::new(0.0, c, s, 0.0),
                Iv4::new(0.0, -s, c, 0.0),
                Iv4::new(0.0, 0.0, 0.0, 1.0),
            ]
        }
    }

    pub fn rotation_y(radians: f32) -> Self {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        Self {
            columns: [
                Iv4::new(c, 0.0, -s, 0.0),
                Iv4::new(0.0, 1.0, 0.0, 0.0),
                Iv4::new(s, 0.0, c, 0.0),
                Iv4::new(0.0, 0.0, 0.0, 1.0),
            ]
        }
    }

    pub fn rotation_z(radians: f32) -> Self {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        Self {
            columns: [
                Iv4::new(c, s, 0.0, 0.0),
                Iv4::new(-s, c, 0.0, 0.0),
                Iv4::new(0.0, 0.0, 1.0, 0.0),
                Iv4::new(0.0, 0.0, 0.0, 1.0),
            ]
        }
    }

    pub fn scale(
        x: impl Into<Fixed<FRACT_BITS>>,
        y: impl Into<Fixed<FRACT_BITS>>,
        z: impl Into<Fixed<FRACT_BITS>>,
    ) -> Self {
        Self {
            columns: [
                Iv4::new(x, 0.0, 0.0, 0.0),
                Iv4::new(0.0, y, 0.0, 0.0),
                Iv4::new(0.0, 0.0, z, 0.0),
                Iv4::new(0.0, 0.0, 0.0, 1.0),
            ]
        }
    }

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> Self {
        let tx = -(right + left) / (right - left);
        let ty = -(top + bottom) / (top - bottom);
        let tz = -(z_far + z_near) / (z_far - z_near);

        Self {
            columns: [
                Iv4::new(2.0 / (right - left), 0.0, 0.0, 0.0),
                Iv4::new(0.0, 2.0 / (top - bottom), 0.0, 0.0),
                Iv4::new(0.0, 0.0, -2.0 / (z_far - z_near), 0.0),
                Iv4::new(tx, ty, tz, 1.0),
            ]
        }
    }

    pub fn perspective(fov_degrees: f32, aspect: f32, z_near: f32, z_far: f32) -> Self {
        let fov_radians = fov_degrees.to_radians();
        let tan = |x| unsafe { intrinsics::sinf32(x) / intrinsics::cosf32(x) };
        let top = z_near * tan(fov_radians / 2.0);
        let right = top * aspect;

        let z_range = z_far - z_near;

        Self {
            columns: [
                Iv4::new(z_near / right, 0.0, 0.0, 0.0),
                Iv4::new(0.0, z_near / top, 0.0, 0.0),
                Iv4::new(0.0, 0.0, -(z_near + z_far) / z_range, -1.0),
                Iv4::new(0.0, 0.0, -2.0 * z_near * z_far / z_range, 0.0),
            ]
        }
    }

    fn rows(&self) -> [Iv4<FRACT_BITS>; 4] {
        [
            Iv4::new(self.columns[0].x, self.columns[1].x, self.columns[2].x, self.columns[3].x),
            Iv4::new(self.columns[0].y, self.columns[1].y, self.columns[2].y, self.columns[3].y),
            Iv4::new(self.columns[0].z, self.columns[1].z, self.columns[2].z, self.columns[3].z),
            Iv4::new(self.columns[0].w, self.columns[1].w, self.columns[2].w, self.columns[3].w),
        ]
    }
}

impl<const FRACT_BITS: u32> Mul for Im4<FRACT_BITS> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        &self * &other
    }
}

impl<'a, const FRACT_BITS: u32> Mul<&'a Self> for Im4<FRACT_BITS> {
    type Output = Self;

    fn mul(self, other: &'a Self) -> Self {
        &self * other
    }
}

impl<'a, const FRACT_BITS: u32> Mul<Im4<FRACT_BITS>> for &'a Im4<FRACT_BITS> {
    type Output = Im4<FRACT_BITS>;

    fn mul(self, other: Im4<FRACT_BITS>) -> Im4<FRACT_BITS> {
        self * &other
    }
}

impl<'a, 'b, const FRACT_BITS: u32> Mul<&'a Im4<FRACT_BITS>> for &'b Im4<FRACT_BITS> {
    type Output = Im4<FRACT_BITS>;

    fn mul(self, other: &'a Im4<FRACT_BITS>) -> Im4<FRACT_BITS> {
        let rows = self.rows();
        Im4 {
            columns: [
                Iv4::new(
                    rows[0].dot(other.columns[0]),
                    rows[1].dot(other.columns[0]),
                    rows[2].dot(other.columns[0]),
                    rows[3].dot(other.columns[0]),
                ),
                Iv4::new(
                    rows[0].dot(other.columns[1]),
                    rows[1].dot(other.columns[1]),
                    rows[2].dot(other.columns[1]),
                    rows[3].dot(other.columns[1]),
                ),
                Iv4::new(
                    rows[0].dot(other.columns[2]),
                    rows[1].dot(other.columns[2]),
                    rows[2].dot(other.columns[2]),
                    rows[3].dot(other.columns[2]),
                ),
                Iv4::new(
                    rows[0].dot(other.columns[3]),
                    rows[1].dot(other.columns[3]),
                    rows[2].dot(other.columns[3]),
                    rows[3].dot(other.columns[3]),
                ),
            ],
        }
    }
}

impl<const FRACT_BITS: u32> Mul<Iv4<FRACT_BITS>> for Im4<FRACT_BITS> {
    type Output = Iv4<FRACT_BITS>;

    fn mul(self, other: Iv4<FRACT_BITS>) -> Iv4<FRACT_BITS> {
        let rows = self.rows();
        Iv4::new(
            rows[0].dot(other),
            rows[1].dot(other),
            rows[2].dot(other),
            rows[3].dot(other),
        )
    }
}

impl<const FRACT_BITS: u32> MulAssign for Im4<FRACT_BITS> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}
