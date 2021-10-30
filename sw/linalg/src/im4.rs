use crate::fixed::*;
use crate::iv4::*;

use core::ops::Mul;

#[derive(Clone, Copy)]
pub struct Im4<const FRACT_BITS: u32> {
    pub columns: [Iv4<FRACT_BITS>; 4],
}

impl<const FRACT_BITS: u32> Im4<FRACT_BITS> {
    pub fn identity() -> Self {
        Self {
            columns: [
                Iv4::new(Fixed::one(), Fixed::zero(), Fixed::zero(), Fixed::zero()),
                Iv4::new(Fixed::zero(), Fixed::one(), Fixed::zero(), Fixed::zero()),
                Iv4::new(Fixed::zero(), Fixed::zero(), Fixed::one(), Fixed::zero()),
                Iv4::new(Fixed::zero(), Fixed::zero(), Fixed::zero(), Fixed::one()),
            ],
        }
    }

    pub fn translation(x: Fixed<FRACT_BITS>, y: Fixed<FRACT_BITS>, z: Fixed<FRACT_BITS>) -> Self {
        Self {
            columns: [
                Iv4::new(Fixed::one(), Fixed::zero(), Fixed::zero(), Fixed::zero()),
                Iv4::new(Fixed::zero(), Fixed::one(), Fixed::zero(), Fixed::zero()),
                Iv4::new(Fixed::zero(), Fixed::zero(), Fixed::one(), Fixed::zero()),
                Iv4::new(x, y, z, Fixed::one()),
            ],
        }
    }

    pub fn scale(x: Fixed<FRACT_BITS>, y: Fixed<FRACT_BITS>, z: Fixed<FRACT_BITS>) -> Self {
        Self {
            columns: [
                Iv4::new(x, Fixed::zero(), Fixed::zero(), Fixed::zero()),
                Iv4::new(Fixed::zero(), y, Fixed::zero(), Fixed::zero()),
                Iv4::new(Fixed::zero(), Fixed::zero(), z, Fixed::zero()),
                Iv4::new(Fixed::zero(), Fixed::zero(), Fixed::zero(), Fixed::one()),
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
