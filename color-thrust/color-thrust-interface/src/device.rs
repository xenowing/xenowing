pub trait Device {
    fn write_reg(&mut self, addr: u32, data: u32);
    fn read_reg(&mut self, addr: u32) -> u32;
    fn write_color_buffer_word(&mut self, addr: u32, data: u128);
    fn read_color_buffer_word(&mut self, addr: u32) -> u128;
    fn write_depth_buffer_word(&mut self, addr: u32, data: u128);
    fn read_depth_buffer_word(&mut self, addr: u32) -> u128;
    fn write_tex_buffer_word(&mut self, addr: u32, data: u128);
}

impl<D: Device + ?Sized> Device for &mut D {
    #[inline]
    fn write_reg(&mut self, addr: u32, data: u32) {
        (**self).write_reg(addr, data);
    }

    #[inline]
    fn read_reg(&mut self, addr: u32) -> u32 {
        (**self).read_reg(addr)
    }

    #[inline]
    fn write_color_buffer_word(&mut self, addr: u32, data: u128) {
        (**self).write_color_buffer_word(addr, data);
    }

    #[inline]
    fn read_color_buffer_word(&mut self, addr: u32) -> u128 {
        (**self).read_color_buffer_word(addr)
    }

    #[inline]
    fn write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        (**self).write_depth_buffer_word(addr, data);
    }

    #[inline]
    fn read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        (**self).read_depth_buffer_word(addr)
    }

    #[inline]
    fn write_tex_buffer_word(&mut self, addr: u32, data: u128) {
        (**self).write_tex_buffer_word(addr, data);
    }
}
