#![no_std]

pub trait Device {
    fn mem_alloc(&mut self, num_words: u32, align_words: u32) -> u32;
    fn mem_dealloc(&mut self, addr: u32);
    fn mem_write_word(&mut self, addr: u32, data: u128);

    fn color_thrust_write_reg(&mut self, addr: u32, data: u32);
    fn color_thrust_read_reg(&mut self, addr: u32) -> u32;
    fn color_thrust_write_color_buffer_word(&mut self, addr: u32, data: u128);
    fn color_thrust_read_color_buffer_word(&mut self, addr: u32) -> u128;
    fn color_thrust_write_depth_buffer_word(&mut self, addr: u32, data: u128);
    fn color_thrust_read_depth_buffer_word(&mut self, addr: u32) -> u128;
}

impl<D: Device + ?Sized> Device for &mut D {
    #[inline]
    fn mem_alloc(&mut self, num_words: u32, align_words: u32) -> u32 {
        (**self).mem_alloc(num_words, align_words)
    }

    #[inline]
    fn mem_dealloc(&mut self, addr: u32) {
        (**self).mem_dealloc(addr);
    }

    #[inline]
    fn mem_write_word(&mut self, addr: u32, data: u128) {
        (**self).mem_write_word(addr, data);
    }

    #[inline]
    fn color_thrust_write_reg(&mut self, addr: u32, data: u32) {
        (**self).color_thrust_write_reg(addr, data);
    }

    #[inline]
    fn color_thrust_read_reg(&mut self, addr: u32) -> u32 {
        (**self).color_thrust_read_reg(addr)
    }

    #[inline]
    fn color_thrust_write_color_buffer_word(&mut self, addr: u32, data: u128) {
        (**self).color_thrust_write_color_buffer_word(addr, data);
    }

    #[inline]
    fn color_thrust_read_color_buffer_word(&mut self, addr: u32) -> u128 {
        (**self).color_thrust_read_color_buffer_word(addr)
    }

    #[inline]
    fn color_thrust_write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        (**self).color_thrust_write_depth_buffer_word(addr, data);
    }

    #[inline]
    fn color_thrust_read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        (**self).color_thrust_read_depth_buffer_word(addr)
    }
}
