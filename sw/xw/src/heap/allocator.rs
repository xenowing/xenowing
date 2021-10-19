use core::alloc::Layout;

pub trait Allocator {
    fn init(&mut self, heap_start: usize, heap_end: usize);

    fn alloc(&mut self, layout: Layout) -> *mut u8;
    fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
}
