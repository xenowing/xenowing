use super::allocator::*;

use core::alloc::Layout;
use core::mem;
use core::ptr::{self, NonNull};

const NUM_SIZE_CLASSES: usize = 8;
const SIZE_CLASS_BLOCK_SIZES: [usize; NUM_SIZE_CLASSES] = [16, 32, 64, 128, 256, 512, 1024, 2048];
const_assert!(verify_size_class_block_sizes());

// This is unused at runtime, but is required by const asserts
#[allow(dead_code)]
const fn verify_size_class_block_sizes() -> bool {
    let mut i = 0;
    while i < NUM_SIZE_CLASSES {
        let block_size = SIZE_CLASS_BLOCK_SIZES[i];

        if !block_size.is_power_of_two() {
            return false;
        }

        if i > 0 {
            let prev_block_size = SIZE_CLASS_BLOCK_SIZES[i - 1];
            if block_size != prev_block_size * 2 {
                return false;
            }
        }

        i += 1;
    }

    true
}

#[repr(C)]
struct FreeBlockHeader {
    next: Option<NonNull<FreeBlockHeader>>,
}

const_assert!(mem::size_of::<FreeBlockHeader>() <= SIZE_CLASS_BLOCK_SIZES[0]);
const_assert!(mem::align_of::<FreeBlockHeader>() <= SIZE_CLASS_BLOCK_SIZES[0]);

pub struct SegregatedListAllocator<T: Allocator> {
    free_heads: [Option<NonNull<FreeBlockHeader>>; NUM_SIZE_CLASSES],
    fallback_allocator: T,
}

impl<T: Allocator> SegregatedListAllocator<T> {
    pub const fn new(fallback_allocator: T) -> SegregatedListAllocator<T> {
        SegregatedListAllocator {
            free_heads: [None; NUM_SIZE_CLASSES],
            fallback_allocator,
        }
    }
}

impl<T: Allocator> Allocator for SegregatedListAllocator<T> {
    fn init(&mut self, heap_start: usize, heap_end: usize) {
        self.fallback_allocator.init(heap_start, heap_end);
    }

    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        match size_class(layout) {
            Some(class) => match self.free_heads[class].take() {
                Some(head) => {
                    let header = unsafe { head.as_ref() };
                    self.free_heads[class] = header.next;
                    head.as_ptr() as *mut _
                }
                _ => {
                    let block_size = SIZE_CLASS_BLOCK_SIZES[class];
                    let block_align = block_size;
                    let layout = unsafe { Layout::from_size_align_unchecked(block_size, block_align) };
                    self.fallback_allocator.alloc(layout)
                }
            }
            _ => self.fallback_allocator.alloc(layout)
        }
    }

    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        match size_class(layout) {
            Some(class) => {
                let header = FreeBlockHeader {
                    next: self.free_heads[class],
                };
                unsafe {
                    ptr::write(ptr as _, header);
                }
                self.free_heads[class] = Some(unsafe { NonNull::new_unchecked(ptr as _) });
            }
            _ => {
                self.fallback_allocator.dealloc(ptr, layout);
            }
        }
    }
}

fn size_class(layout: Layout) -> Option<usize> {
    let block_size = layout.size().max(layout.align());
    SIZE_CLASS_BLOCK_SIZES.iter().position(|&size| size >= block_size)
}
