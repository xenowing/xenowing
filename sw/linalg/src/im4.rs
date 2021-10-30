use crate::iv4::*;

#[derive(Clone, Copy)]
pub struct Im4 {
    pub columns: [Iv4; 4],
}

impl Im4 {
    pub fn identity(shift: u32) -> Im4 {
        Im4 {
            columns: [
                Iv4::new(1 << shift, 0, 0, 0),
                Iv4::new(0, 1 << shift, 0, 0),
                Iv4::new(0, 0, 1 << shift, 0),
                Iv4::new(0, 0, 0, 1 << shift),
            ],
        }
    }

    pub fn translation(x: i32, y: i32, z: i32, shift: u32) -> Im4 {
        Im4 {
            columns: [
                Iv4::new(1 << shift, 0, 0, 0),
                Iv4::new(0, 1 << shift, 0, 0),
                Iv4::new(0, 0, 1 << shift, 0),
                Iv4::new(x, y, z, 1 << shift),
            ],
        }
    }

    pub fn scale(x: i32, y: i32, z: i32, shift: u32) -> Im4 {
        Im4 {
            columns: [
                Iv4::new(x, 0, 0, 0),
                Iv4::new(0, y, 0, 0),
                Iv4::new(0, 0, z, 0),
                Iv4::new(0, 0, 0, 1 << shift),
            ]
        }
    }

    fn rows(&self) -> [Iv4; 4] {
        [
            Iv4::new(self.columns[0].x, self.columns[1].x, self.columns[2].x, self.columns[3].x),
            Iv4::new(self.columns[0].y, self.columns[1].y, self.columns[2].y, self.columns[3].y),
            Iv4::new(self.columns[0].z, self.columns[1].z, self.columns[2].z, self.columns[3].z),
            Iv4::new(self.columns[0].w, self.columns[1].w, self.columns[2].w, self.columns[3].w),
        ]
    }

    pub fn mul_im4(self, other: Im4, shift: u32) -> Im4 {
        let rows = self.rows();
        Im4 {
            columns: [
                Iv4::new(
                    rows[0].dot(other.columns[0], shift),
                    rows[1].dot(other.columns[0], shift),
                    rows[2].dot(other.columns[0], shift),
                    rows[3].dot(other.columns[0], shift),
                ),
                Iv4::new(
                    rows[0].dot(other.columns[1], shift),
                    rows[1].dot(other.columns[1], shift),
                    rows[2].dot(other.columns[1], shift),
                    rows[3].dot(other.columns[1], shift),
                ),
                Iv4::new(
                    rows[0].dot(other.columns[2], shift),
                    rows[1].dot(other.columns[2], shift),
                    rows[2].dot(other.columns[2], shift),
                    rows[3].dot(other.columns[2], shift),
                ),
                Iv4::new(
                    rows[0].dot(other.columns[3], shift),
                    rows[1].dot(other.columns[3], shift),
                    rows[2].dot(other.columns[3], shift),
                    rows[3].dot(other.columns[3], shift),
                ),
            ],
        }
    }

    pub fn mul_iv4(self, other: Iv4, shift: u32) -> Iv4 {
        let rows = self.rows();
        Iv4::new(
            rows[0].dot(other, shift),
            rows[1].dot(other, shift),
            rows[2].dot(other, shift),
            rows[3].dot(other, shift),
        )
    }

    pub fn mul_s(self, other: i32, shift: u32) -> Im4 {
        Im4 {
            columns: [
                self.columns[0].mul_s(other, shift),
                self.columns[1].mul_s(other, shift),
                self.columns[2].mul_s(other, shift),
                self.columns[3].mul_s(other, shift),
            ],
        }
    }
}
