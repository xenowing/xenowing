pub trait Device {
    fn write_reg(&mut self, addr: u32, data: u32);
    fn read_reg(&mut self, addr: u32) -> u32;
    fn write_color_buffer_word(&mut self, addr: u32, data: u32);
    fn read_color_buffer_word(&mut self, addr: u32) -> u32;
    fn write_depth_buffer_word(&mut self, addr: u32, data: u16);
    fn read_depth_buffer_word(&mut self, addr: u32) -> u16;
    fn write_tex_buffer_word(&mut self, addr: u32, data: u128);
}
