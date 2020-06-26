use std::arch::x86::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub};

#[derive(Clone, Copy)]
pub struct Vec4 {
    inner: __m128,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_set_ps(w, z, y, x) },
        }
    }

    pub fn splat(value: f32) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_set1_ps(value) },
        }
    }

    pub fn zero() -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_setzero_ps() },
        }
    }

    pub fn x(&self) -> f32 {
        unsafe { _mm_cvtss_f32(self.inner) }
    }

    pub fn y(&self) -> f32 {
        unsafe { _mm_cvtss_f32(_mm_shuffle_ps(self.inner, self.inner, _MM_SHUFFLE(0, 0, 0, 1) as _)) }
    }

    pub fn z(&self) -> f32 {
        unsafe { _mm_cvtss_f32(_mm_shuffle_ps(self.inner, self.inner, _MM_SHUFFLE(0, 0, 0, 2) as _)) }
    }

    pub fn w(&self) -> f32 {
        unsafe { _mm_cvtss_f32(_mm_shuffle_ps(self.inner, self.inner, _MM_SHUFFLE(0, 0, 0, 3) as _)) }
    }

    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Vec4 {
        let len = self.len();
        self / len
    }

    pub fn dot(self, other: Vec4) -> f32 {
        unsafe {
            let mut product = _mm_mul_ps(self.inner, other.inner);
            product = _mm_hadd_ps(product, product);
            product = _mm_hadd_ps(product, product);
            _mm_cvtss_f32(product)
        }
    }

    pub fn min(self, other: Vec4) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_min_ps(self.inner, other.inner) },
        }
    }

    pub fn max(self, other: Vec4) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_max_ps(self.inner, other.inner) },
        }
    }
}

impl Add for Vec4 {
    type Output = Vec4;

    fn add(self, other: Vec4) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_add_ps(self.inner, other.inner) },
        }
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, other: Vec4) {
        *self = *self + other
    }
}

impl Div for Vec4 {
    type Output = Vec4;

    fn div(self, other: Vec4) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_div_ps(self.inner, other.inner) },
        }
    }
}

impl Div<f32> for Vec4 {
    type Output = Vec4;

    fn div(self, other: f32) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_div_ps(self.inner, _mm_set1_ps(other)) },
        }
    }
}

impl DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other
    }
}

impl Mul for Vec4 {
    type Output = Vec4;

    fn mul(self, other: Vec4) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_mul_ps(self.inner, other.inner) },
        }
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;

    fn mul(self, other: f32) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_mul_ps(self.inner, _mm_set1_ps(other)) },
        }
    }
}

impl Sub for Vec4 {
    type Output = Vec4;

    fn sub(self, other: Vec4) -> Vec4 {
        Vec4 {
            inner: unsafe { _mm_sub_ps(self.inner, other.inner) },
        }
    }
}
