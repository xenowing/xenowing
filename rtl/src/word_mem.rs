use kaze::*;

pub struct WordMem<'a> {
    element_bit_width: u32,

    mems: Vec<&'a Mem<'a>>,
}

impl<'a> WordMem<'a> {
    pub fn new(module: &'a Module<'a>, name: impl Into<String>, addr_bit_width: u32, element_bit_width: u32, elements_per_word: u32) -> WordMem<'a> {
        let name = name.into();

        WordMem {
            element_bit_width,

            mems: (0..elements_per_word).map(|element_index| {
                module.mem(
                    format!("{}_element_{}", name, element_index),
                    addr_bit_width,
                    element_bit_width)
            }).collect(),
        }
    }

    pub fn read_port(&self, address: &'a dyn Signal<'a>, enable: &'a dyn Signal<'a>) -> &'a dyn Signal<'a> {
        let mut ret = None;

        for mem in self.mems.iter() {
            let read_element = mem.read_port(address, enable);
            ret = Some(if let Some(word) = ret {
                read_element.concat(word)
            } else {
                read_element
            });
        }

        ret.unwrap()
    }

    pub fn write_port(&self, address: &'a dyn Signal<'a>, value: &'a dyn Signal<'a>, enable: &'a dyn Signal<'a>, element_enable: &'a dyn Signal<'a>) {
        for (element_index, mem) in self.mems.iter().enumerate() {
            let element_index = element_index as u32;
            let element_word_offset_bits = element_index * self.element_bit_width;
            let element_write_data = value.bits(element_word_offset_bits + self.element_bit_width - 1, element_word_offset_bits);
            let element_write_enable = enable & element_enable.bit(element_index);
            mem.write_port(address, element_write_data, element_write_enable);
        }
    }
}
