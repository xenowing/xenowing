// TODO: Dedupe and move!
pub const MEM_ADDR_BITS: u32 = 24;
pub const MEM_NUM_WORDS: u32 = 1 << MEM_ADDR_BITS;
pub const MEM_NUM_BYTES: u32 = MEM_NUM_WORDS << 4;

struct Allocation {
    start: u32,
    size: u32,
}

impl Allocation {
    fn end(&self) -> u32 {
        self.start + self.size
    }
}

pub struct MemAllocator {
    allocations: Vec<Allocation>,
}

impl MemAllocator {
    pub fn new() -> MemAllocator {
        MemAllocator {
            allocations: Vec::new(),
        }
    }

    pub fn alloc(&mut self, num_words: u32, align_words: u32) -> u32 {
        let size = num_words * 16;
        let align = align_words * 16;
        let mut start = 0;
        let mut end = start + size;
        let mut insert_index = 0;
        for (i, allocation) in self.allocations.iter().enumerate() {
            let allocation_end = allocation.end();
            if (start >= allocation.start && start < allocation_end) ||
                (end >= allocation.start && end < allocation_end) ||
                (allocation.start >= start && allocation.start < end) ||
                (allocation_end >= start && allocation_end < end) {
                start = (allocation_end + align - 1) / align * align;
                end = start + size;
                insert_index = i + 1;
            }
        }
        if end >= MEM_NUM_BYTES {
            panic!("Out of device memory");
        }
        self.allocations.insert(insert_index, Allocation {
            start,
            size,
        });
        start
    }

    pub fn dealloc(&mut self, _addr: u32) {
        todo!()
    }
}
