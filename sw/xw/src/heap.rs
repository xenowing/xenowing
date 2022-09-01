mod allocator;
mod explicit_free_list_allocator;
mod segregated_list_allocator;

use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;

use allocator::*;
use explicit_free_list_allocator::*;
use segregated_list_allocator::*;

#[global_allocator]
static mut ALLOCATOR: SystemAllocator<SegregatedListAllocator<ExplicitFreeListAllocator>> =
    SystemAllocator::new(SegregatedListAllocator::new(ExplicitFreeListAllocator::new()));

pub fn init() {
    unsafe {
        ALLOCATOR.init();
    }
}

struct SystemAllocator<T: Allocator> {
    impl_: RefCell<T>,
}

impl<T: Allocator> SystemAllocator<T> {
    const fn new(impl_: T) -> SystemAllocator<T> {
        SystemAllocator {
            impl_: RefCell::new(impl_),
        }
    }

    fn init(&mut self) {
        extern "C" {
            static _sheap: u8;
            static _eheap: u8;
        }

        let heap_start = unsafe { &_sheap } as *const _ as usize;
        let heap_end = unsafe { &_eheap } as *const _ as usize;

        self.impl_.borrow_mut().init(heap_start, heap_end);
    }
}

unsafe impl<T: Allocator> GlobalAlloc for SystemAllocator<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.impl_.borrow_mut().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.impl_.borrow_mut().dealloc(ptr, layout);
    }
}
