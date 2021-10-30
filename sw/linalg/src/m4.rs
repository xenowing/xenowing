use crate::v4::*;

use core::intrinsics;
use core::ops::Mul;

#[derive(Clone, Copy)]
pub struct M4 {
    pub columns: [V4; 4],
}

impl M4 {
    pub fn identity() -> M4 {
        M4 {
            columns: [
                V4::new(1.0, 0.0, 0.0, 0.0),
                V4::new(0.0, 1.0, 0.0, 0.0),
                V4::new(0.0, 0.0, 1.0, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            ],
        }
    }

    pub fn translation(x: f32, y: f32, z: f32) -> M4 {
        M4 {
            columns: [
                V4::new(1.0, 0.0, 0.0, 0.0),
                V4::new(0.0, 1.0, 0.0, 0.0),
                V4::new(0.0, 0.0, 1.0, 0.0),
                V4::new(x, y, z, 1.0),
            ],
        }
    }

    pub fn rotation_x(radians: f32) -> M4 {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        M4 {
            columns: [
                V4::new(1.0, 0.0, 0.0, 0.0),
                V4::new(0.0, c, s, 0.0),
                V4::new(0.0, -s, c, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            ]
        }
    }

    pub fn rotation_y(radians: f32) -> M4 {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        M4 {
            columns: [
                V4::new(c, 0.0, -s, 0.0),
                V4::new(0.0, 1.0, 0.0, 0.0),
                V4::new(s, 0.0, c, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            ]
        }
    }

    pub fn rotation_z(radians: f32) -> M4 {
        let s = unsafe { intrinsics::sinf32(radians) };
        let c = unsafe { intrinsics::cosf32(radians) };

        M4 {
            columns: [
                V4::new(c, s, 0.0, 0.0),
                V4::new(-s, c, 0.0, 0.0),
                V4::new(0.0, 0.0, 1.0, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            ]
        }
    }

    pub fn scale(x: f32, y: f32, z: f32) -> M4 {
        M4 {
            columns: [
                V4::new(x, 0.0, 0.0, 0.0),
                V4::new(0.0, y, 0.0, 0.0),
                V4::new(0.0, 0.0, z, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            ]
        }
    }

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> M4 {
        let tx = -(right + left) / (right - left);
        let ty = -(top + bottom) / (top - bottom);
        let tz = -(z_far + z_near) / (z_far - z_near);

        M4 {
            columns: [
                V4::new(2.0 / (right - left), 0.0, 0.0, 0.0),
                V4::new(0.0, 2.0 / (top - bottom), 0.0, 0.0),
                V4::new(0.0, 0.0, -2.0 / (z_far - z_near), 0.0),
                V4::new(tx, ty, tz, 1.0),
            ]
        }
    }

    pub fn perspective(fov_degrees: f32, aspect: f32, z_near: f32, z_far: f32) -> M4 {
        let fov_radians = fov_degrees.to_radians();
        let tan = |x| unsafe { intrinsics::sinf32(x) / intrinsics::cosf32(x) };
        let top = z_near * tan(fov_radians / 2.0);
        let right = top * aspect;

        let z_range = z_far - z_near;

        M4 {
            columns: [
                V4::new(z_near / right, 0.0, 0.0, 0.0),
                V4::new(0.0, z_near / top, 0.0, 0.0),
                V4::new(0.0, 0.0, -(z_near + z_far) / z_range, -1.0),
                V4::new(0.0, 0.0, -2.0 * z_near * z_far / z_range, 0.0),
            ]
        }
    }

    fn rows(&self) -> [V4; 4] {
        [
            V4::new(self.columns[0].x, self.columns[1].x, self.columns[2].x, self.columns[3].x),
            V4::new(self.columns[0].y, self.columns[1].y, self.columns[2].y, self.columns[3].y),
            V4::new(self.columns[0].z, self.columns[1].z, self.columns[2].z, self.columns[3].z),
            V4::new(self.columns[0].w, self.columns[1].w, self.columns[2].w, self.columns[3].w),
        ]
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
        let rows = self.rows();
        M4 {
            columns: [
                V4::new(
                    rows[0].dot(other.columns[0]),
                    rows[1].dot(other.columns[0]),
                    rows[2].dot(other.columns[0]),
                    rows[3].dot(other.columns[0]),
                ),
                V4::new(
                    rows[0].dot(other.columns[1]),
                    rows[1].dot(other.columns[1]),
                    rows[2].dot(other.columns[1]),
                    rows[3].dot(other.columns[1]),
                ),
                V4::new(
                    rows[0].dot(other.columns[2]),
                    rows[1].dot(other.columns[2]),
                    rows[2].dot(other.columns[2]),
                    rows[3].dot(other.columns[2]),
                ),
                V4::new(
                    rows[0].dot(other.columns[3]),
                    rows[1].dot(other.columns[3]),
                    rows[2].dot(other.columns[3]),
                    rows[3].dot(other.columns[3]),
                ),
            ],
        }
    }
}

impl Mul<V4> for M4 {
    type Output = V4;

    fn mul(self, other: V4) -> V4 {
        let rows = self.rows();
        V4::new(
            rows[0].dot(other),
            rows[1].dot(other),
            rows[2].dot(other),
            rows[3].dot(other),
        )
    }
}
