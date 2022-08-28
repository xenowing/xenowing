#![no_std]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

const NUM_PREV_PIXELS_ENTRIES: usize = 64;

const QOI_OP_RGB: u8 = 0b11111110;
const QOI_OP_RGBA: u8 = 0b11111111;
const QOI_OP_INDEX: u8 = 0b00000000;
const QOI_OP_DIFF: u8 = 0b01000000;
const QOI_OP_LUMA: u8 = 0b10000000;
const QOI_OP_RUN: u8 = 0b11000000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Pixel(u32);

impl Pixel {
    pub fn from_components(a: u8, r: u8, g: u8, b: u8) -> Pixel {
        Pixel(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0))
    }

    pub fn to_argb(&self) -> u32 {
        self.0
    }

    pub fn a(&self) -> u8 {
        (self.0 >> 24) as _
    }

    pub fn r(&self) -> u8 {
        (self.0 >> 16) as _
    }

    pub fn g(&self) -> u8 {
        (self.0 >> 8) as _
    }

    pub fn b(&self) -> u8 {
        (self.0 >> 0) as _
    }

    fn prev_pixels_index(&self) -> usize {
        self.r().wrapping_mul(3)
            .wrapping_add(self.g().wrapping_mul(5))
            .wrapping_add(self.b().wrapping_mul(7))
            .wrapping_add(self.a().wrapping_mul(11))
            as usize % NUM_PREV_PIXELS_ENTRIES
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Box<[Pixel]>,
}

impl Image {
    pub fn encode(&self) -> Box<[u8]> {
        let mut ret = Vec::new();

        // Header
        ret.extend_from_slice(b"qoif"); // magic
        ret.extend_from_slice(&self.width.to_be_bytes());
        ret.extend_from_slice(&self.height.to_be_bytes());
        ret.push(4); // channels
        ret.push(0); // colorspace (sRGB with linear alpha)

        // Data
        let mut prev_pixels = PrevPixels::new();

        let mut i = 0;
        while i < self.width * self.height {
            let pixel = self.data[i as usize];

            let mut run_length = 0;
            while i + run_length < self.width * self.height && self.data[(i + run_length) as usize] == prev_pixels.prev && run_length < 62 {
                run_length += 1;
            }

            if run_length >= 1 {
                ret.push(QOI_OP_RUN | (run_length as u8 - 1));
                i += run_length;
                continue;
            }

            let index = pixel.prev_pixels_index();
            if prev_pixels.entries[index] == pixel {
                ret.push(QOI_OP_INDEX | index as u8);
            } else if pixel.a() == prev_pixels.prev.a() {
                let r_diff = pixel.r().wrapping_sub(prev_pixels.prev.r()) as i8;
                let g_diff = pixel.g().wrapping_sub(prev_pixels.prev.g()) as i8;
                let b_diff = pixel.b().wrapping_sub(prev_pixels.prev.b()) as i8;
                let rg_diff = r_diff.wrapping_sub(g_diff);
                let bg_diff = b_diff.wrapping_sub(g_diff);
                if r_diff >= -2 && r_diff <= 1 &&
                    g_diff >= -2 && g_diff <= 1 &&
                    b_diff >= -2 && b_diff <= 1 {
                    ret.push(QOI_OP_DIFF |
                        (((r_diff + 2) as u8) << 4) |
                        (((g_diff + 2) as u8) << 2) |
                        (((b_diff + 2) as u8) << 0));
                } else if g_diff >= -32 && g_diff <= 31 &&
                    rg_diff >= -8 && rg_diff <= 7 &&
                    bg_diff >= -8 && bg_diff <= 7 {
                    ret.push(QOI_OP_LUMA | ((g_diff + 32) as u8));
                    ret.push((((rg_diff + 8) as u8) << 4) | (((bg_diff + 8) as u8) << 0));
                } else {
                    ret.push(QOI_OP_RGB);
                    ret.push(pixel.r());
                    ret.push(pixel.g());
                    ret.push(pixel.b());
                }
            } else {
                ret.push(QOI_OP_RGBA);
                ret.push(pixel.r());
                ret.push(pixel.g());
                ret.push(pixel.b());
                ret.push(pixel.a());
            }

            prev_pixels.insert(pixel);

            i += 1;
        }

        // Stream end
        for _ in 0..7 {
            ret.push(0x00);
        }
        ret.push(0x01);

        ret.into_boxed_slice()
    }

    pub fn decode(input: &[u8]) -> Image {
        let mut cursor = Cursor::new(input);

        // Header
        assert_eq!(&cursor.read_u32_be().to_be_bytes(), b"qoif"); // magic
        let width = cursor.read_u32_be();
        let height = cursor.read_u32_be();
        assert_eq!(cursor.read_u8(), 4); // channels
        assert_eq!(cursor.read_u8(), 0); // colorspace

        // Data
        let mut data = Vec::with_capacity((width * height) as _);

        let mut prev_pixels = PrevPixels::new();

        while (data.len() as u32) < width * height {
            let tag_byte = cursor.read_u8();
            if tag_byte == QOI_OP_RGB {
                let r = cursor.read_u8();
                let g = cursor.read_u8();
                let b = cursor.read_u8();
                let a = prev_pixels.prev.a();
                let pixel = Pixel::from_components(a, r, g, b);
                data.push(pixel);
                prev_pixels.insert(pixel);
            } else if tag_byte == QOI_OP_RGBA {
                let r = cursor.read_u8();
                let g = cursor.read_u8();
                let b = cursor.read_u8();
                let a = cursor.read_u8();
                let pixel = Pixel::from_components(a, r, g, b);
                data.push(pixel);
                prev_pixels.insert(pixel);
            } else {
                match tag_byte & 0b11000000 {
                    QOI_OP_INDEX => {
                        let index = (tag_byte & 0b00111111) as usize;
                        let pixel = prev_pixels.entries[index];
                        data.push(pixel);
                        prev_pixels.insert(pixel);
                    }
                    QOI_OP_DIFF => {
                        let r_diff = ((tag_byte >> 4) & 0b11).wrapping_sub(2);
                        let g_diff = ((tag_byte >> 2) & 0b11).wrapping_sub(2);
                        let b_diff = ((tag_byte >> 0) & 0b11).wrapping_sub(2);
                        let r = prev_pixels.prev.r().wrapping_add(r_diff);
                        let g = prev_pixels.prev.g().wrapping_add(g_diff);
                        let b = prev_pixels.prev.b().wrapping_add(b_diff);
                        let a = prev_pixels.prev.a();
                        let pixel = Pixel::from_components(a, r, g, b);
                        data.push(pixel);
                        prev_pixels.insert(pixel);
                    }
                    QOI_OP_LUMA => {
                        let g_diff = (tag_byte & 0b00111111).wrapping_sub(32);
                        let diffs_byte = cursor.read_u8();
                        let rg_diff = ((diffs_byte >> 4) & 0b1111).wrapping_sub(8);
                        let bg_diff = ((diffs_byte >> 0) & 0b1111).wrapping_sub(8);
                        let g = prev_pixels.prev.g().wrapping_add(g_diff);
                        let r = prev_pixels.prev.r().wrapping_add(g_diff).wrapping_add(rg_diff);
                        let b = prev_pixels.prev.b().wrapping_add(g_diff).wrapping_add(bg_diff);
                        let a = prev_pixels.prev.a();
                        let pixel = Pixel::from_components(a, r, g, b);
                        data.push(pixel);
                        prev_pixels.insert(pixel);
                    }
                    QOI_OP_RUN => {
                        let length = (tag_byte & 0b00111111) + 1;
                        for _ in 0..length {
                            data.push(prev_pixels.prev);
                        }
                    }
                    _ => panic!("Unrecognized tag format")
                }
            };
        }

        assert!(data.len() as u32 == width * height);

        // Stream end
        for _ in 0..7 {
            assert_eq!(cursor.read_u8(), 0x00);
        }
        assert_eq!(cursor.read_u8(), 0x01);

        Image {
            width,
            height,
            data: data.into_boxed_slice(),
        }
    }
}

struct Cursor<'a> {
    buffer: &'a [u8],
    index: usize,
}

impl<'a> Cursor<'a> {
    fn new(buffer: &'a [u8]) -> Cursor<'a> {
        Cursor {
            buffer,
            index: 0,
        }
    }

    fn read_u8(&mut self) -> u8 {
        let ret = self.buffer[self.index];
        self.index += 1;
        ret
    }

    fn read_u32_be(&mut self) -> u32 {
        let mut ret = 0;
        for i in 0..4 {
            ret |= (self.read_u8() as u32) << ((3 - i) * 8);
        }
        ret
    }
}

struct PrevPixels {
    prev: Pixel,
    entries: Box<[Pixel]>,
}

impl PrevPixels {
    fn new() -> PrevPixels {
        PrevPixels {
            prev: Pixel::from_components(255, 0, 0, 0),
            entries: vec![Pixel::default(); NUM_PREV_PIXELS_ENTRIES].into_boxed_slice(),
        }
    }

    fn insert(&mut self, pixel: Pixel) {
        self.prev = pixel;
        self.entries[pixel.prev_pixels_index()] = pixel;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use image::GenericImageView;

    use alloc::vec;

    use std::path::Path;

    fn round_trip_test<P: AsRef<Path>>(path: P) {
        let input = {
            let input = image::open(path).expect("Couldn't load image");
            let width = input.width();
            let height = input.height();
            let mut data = vec![Pixel::default(); (width * height) as usize].into_boxed_slice();
            for y in 0..height {
                for x in 0..width {
                    let pixel = input.get_pixel(x, y);
                    let r = pixel[0];
                    let g = pixel[1];
                    let b = pixel[2];
                    let a = pixel[3];
                    data[(y * width + x) as usize] = Pixel::from_components(a, r, g, b);
                }
            }
            Image {
                width,
                height,
                data,
            }
        };

        println!("Uncompressed size: {} bytes", input.data.len() * 4);

        let encoded = input.encode();
        println!("Compressed size: {} bytes", encoded.len());
        let decoded = Image::decode(&encoded);

        assert_eq!(decoded, input);
    }

    #[test]
    fn my_face() {
        round_trip_test("myface.png");
    }

    #[test]
    fn white() {
        round_trip_test("white.png");
    }

    #[test]
    fn tex() {
        round_trip_test("tex.png");
    }

    #[test]
    fn tex2() {
        round_trip_test("tex2.png");
    }
}
