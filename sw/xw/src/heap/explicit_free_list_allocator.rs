use super::allocator::*;

//use crate::stdio;

use core::alloc::Layout;
//use core::fmt::Write;
use core::mem;
use core::ptr::{self, NonNull};

#[derive(Debug)]
#[repr(C)]
struct BlockHeader {
    storage: usize,
}

impl BlockHeader {
    fn new(block_end: usize, is_prev_allocated: bool) -> BlockHeader {
        BlockHeader {
            storage: block_end | ((is_prev_allocated as usize) << 1),
        }
    }

    // Explicit lifetimes to work around the compiler inferring that `self` must outlive the return value
    fn additional_header<'a, 'b>(&'a self) -> &'b mut FreeBlockAdditionalHeader {
        let additional_header_ptr =
            (self.block_start() + mem::size_of::<Self>()) as *mut FreeBlockAdditionalHeader;
        unsafe { &mut *additional_header_ptr }
    }

    fn block_start(&self) -> usize {
        self as *const _ as _
    }

    fn block_end(&self) -> usize {
        self.storage & !3
    }

    fn clear_allocated(&mut self) {
        self.storage &= !(1 << 0);
    }

    fn clear_prev_allocated(&mut self) {
        self.storage &= !(1 << 1);
    }

    fn end(&self) -> usize {
        self.start() + mem::size_of::<Self>()
    }

    /*fn footer(&self) -> &mut FreeBlockFooter {
        let footer_ptr = (self.block_end() - mem::size_of::<FreeBlockFooter>()) as *mut FreeBlockFooter;
        unsafe { &mut *footer_ptr }
    }*/

    fn is_allocated(&self) -> bool {
        (self.storage & (1 << 0)) != 0
    }

    fn is_prev_allocated(&self) -> bool {
        (self.storage & (1 << 1)) != 0
    }

    fn set_allocated(&mut self) {
        self.storage |= 1 << 0;
    }

    fn set_block_end(&mut self, block_end: usize) {
        self.storage = block_end | (self.storage & 3);
    }

    fn set_prev_allocated(&mut self) {
        self.storage |= 1 << 1;
    }

    fn start(&self) -> usize {
        self as *const _ as _
    }
}

// TODO: Consider typestates for encoding which fields are present in which blocks for added safety
#[derive(Debug)]
#[repr(C)]
struct FreeBlockAdditionalHeader {
    free_prev: Option<NonNull<BlockHeader>>,
    free_next: Option<NonNull<BlockHeader>>,
}

#[derive(Debug)]
#[repr(C)]
struct FreeBlockFooter {
    header: NonNull<BlockHeader>,
}

const MIN_FREE_BLOCK_SIZE: usize = mem::size_of::<BlockHeader>()
    + mem::size_of::<FreeBlockAdditionalHeader>()
    + mem::size_of::<FreeBlockFooter>();

// This is unused at runtime, but is required by const asserts
#[allow(dead_code)]
const MIN_ALLOCATED_BLOCK_SIZE: usize =
    mem::size_of::<BlockHeader>() + mem::size_of::<NonNull<BlockHeader>>();

const MIN_BLOCK_SIZE: usize = MIN_FREE_BLOCK_SIZE;

const_assert!(mem::align_of::<BlockHeader>() >= 4);
const_assert_eq!(
    mem::align_of::<BlockHeader>(),
    mem::align_of::<FreeBlockAdditionalHeader>()
);
const_assert_eq!(
    mem::align_of::<BlockHeader>(),
    mem::align_of::<FreeBlockFooter>()
);
const_assert_eq!(
    mem::align_of::<BlockHeader>(),
    mem::align_of::<NonNull<BlockHeader>>()
);
const_assert!(MIN_FREE_BLOCK_SIZE >= MIN_ALLOCATED_BLOCK_SIZE);

pub struct ExplicitFreeListAllocator {
    free_head: Option<NonNull<BlockHeader>>,
    heap_start: usize,
    heap_end: usize,
}

impl ExplicitFreeListAllocator {
    pub const fn new() -> ExplicitFreeListAllocator {
        ExplicitFreeListAllocator {
            free_head: None,
            heap_start: 0,
            heap_end: 0,
        }
    }

    fn push_onto_free_list(&mut self, block: &mut BlockHeader) {
        let additional_header = block.additional_header();
        additional_header.free_prev = None;
        additional_header.free_next = self.free_head;

        if let Some(mut free_head_ptr) = self.free_head {
            let free_head = unsafe { free_head_ptr.as_mut() };
            free_head.additional_header().free_prev = Some(block.into());
        }

        self.free_head = Some(block.into());
    }

    fn splice_out_of_free_list(&mut self, block: &mut FreeBlockAdditionalHeader) {
        if let Some(mut free_prev_ptr) = block.free_prev {
            let free_prev = unsafe { free_prev_ptr.as_mut() };
            let free_prev = free_prev.additional_header();
            // This block is in the tail of the free list; patch free_prev and free_next
            free_prev.free_next = block.free_next;
            if let Some(mut free_next_ptr) = block.free_next {
                let free_next = unsafe { free_next_ptr.as_mut() };
                let free_next = free_next.additional_header();
                free_next.free_prev = block.free_prev;
            }
            block.free_prev = None;
        } else {
            // This block is the head of the free list; remove it, and push free_next in its place
            self.free_head = if let Some(mut free_next_ptr) = block.free_next {
                let free_next = unsafe { free_next_ptr.as_mut() };
                free_next.additional_header().free_prev = None;

                Some(free_next.into())
            } else {
                None
            };
        }
        block.free_next = None;
    }

    fn coalesce_blocks(&mut self, header: &mut BlockHeader, next_header: &mut BlockHeader) {
        //writeln!(stdio::stdout(), "   coalescing blocks:").unwrap();
        //writeln!(stdio::stdout(), "       {:?}: {:?} + {:?} + {:?}", header as *mut _, header, header.additional_header(), header.footer()).unwrap();
        //writeln!(stdio::stdout(), "       {:?}: {:?} + {:?} + {:?}", next_header as *mut _, next_header, next_header.additional_header(), next_header.footer()).unwrap();

        self.splice_out_of_free_list(next_header.additional_header());

        header.set_block_end(next_header.block_end());
        initialize_footer(header.into(), header.block_end());

        //writeln!(stdio::stdout(), "    -> {:?}: {:?} + {:?} + {:?}", header as *mut _, header, header.additional_header(), header.footer()).unwrap();
        //writeln!(stdio::stdout(), "       free head: {:?}", self.free_head).unwrap();
    }
}

impl Allocator for ExplicitFreeListAllocator {
    fn init(&mut self, heap_start: usize, heap_end: usize) {
        assert!(
            self.free_head.is_none(),
            "Heap has already been initialized"
        );
        assert!(heap_end >= heap_start, "Heap end is less than heap start");
        assert!(
            heap_end - heap_start >= MIN_BLOCK_SIZE,
            "Heap is not large enough to fit the minimum block size"
        );
        assert_eq!(
            heap_start % mem::align_of::<BlockHeader>(),
            0,
            "Heap start is not aligned to the block header alignment ({})",
            mem::align_of::<BlockHeader>()
        );
        assert_eq!(
            heap_end % mem::align_of::<BlockHeader>(),
            0,
            "Heap end is not aligned to the block header alignment ({})",
            mem::align_of::<BlockHeader>()
        );

        //writeln!(stdio::stdout(), "heap start: 0x{:08x}", heap_start).unwrap();
        //writeln!(stdio::stdout(), "heap end:   0x{:08x}", heap_end).unwrap();

        self.free_head = Some(initialize_free_block(heap_start, heap_end, false));
        self.heap_start = heap_start;
        self.heap_end = heap_end;
    }

    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        //writeln!(stdio::stdout(), " > alloc with layout: {:?}, free head: {:?}", layout, self.free_head).unwrap();

        let header_pointer_required = layout.align() > mem::align_of::<BlockHeader>();
        //writeln!(stdio::stdout(), "   header pointer required: {}", header_pointer_required).unwrap();

        let mut block_ptr = self.free_head;
        loop {
            let mut header_ptr = match block_ptr {
                Some(header_ptr) => header_ptr,
                _ => {
                    // Out of memory
                    return ptr::null_mut();
                }
            };
            let header = unsafe { header_ptr.as_mut() };
            let additional_header = header.additional_header();

            //writeln!(stdio::stdout(), "   visiting {:?}: {:?} + {:?} + {:?} of size {}", header_ptr, header, additional_header, header.footer(), header.block_end() - header.block_start()).unwrap();

            let prev_header_ptr = header_ptr;
            block_ptr = additional_header.free_next;

            let min_payload_start = header.end()
                + if header_pointer_required {
                    mem::size_of::<NonNull<BlockHeader>>()
                } else {
                    0
                };

            // TODO: Do we need to handle a possible overflow here?
            let payload_start = align_up(min_payload_start, layout.align());
            let payload_end = match payload_start.checked_add(layout.size()) {
                Some(end) => end,
                _ => continue,
            };

            if payload_end > header.block_end() {
                continue;
            }

            // We have a match!
            //writeln!(stdio::stdout(), "   match!").unwrap();

            // Set allocated flag(s)
            header.set_allocated();
            let next_block_start = header.block_end();
            if next_block_start != self.heap_end {
                let mut next_block_ptr =
                    unsafe { NonNull::new_unchecked(next_block_start as *mut BlockHeader) };
                let next_header = unsafe { next_block_ptr.as_mut() };
                next_header.set_prev_allocated();
            }

            self.splice_out_of_free_list(additional_header);

            // Write header pointer
            if header_pointer_required {
                let header_pointer = (payload_start - mem::size_of::<NonNull<BlockHeader>>()) as _;
                //writeln!(stdio::stdout(), "   header pointer: {:?}", header_pointer).unwrap();
                unsafe {
                    ptr::write(header_pointer, prev_header_ptr);
                }
            }

            // If possible, split the current block so unused space will be covered by a new block
            let min_new_block_start = header.block_start() + MIN_BLOCK_SIZE;
            let new_block_start = payload_end.max(min_new_block_start);
            if new_block_start < header.block_end()
                && header.block_end() - new_block_start >= MIN_BLOCK_SIZE
            {
                //writeln!(stdio::stdout(), "   splitting block; new block start: {:?}", new_block_start as *mut u8).unwrap();
                // Build new header for unused block, splice into block list
                let mut new_header_ptr =
                    initialize_free_block(new_block_start, header.block_end(), true);
                let new_header = unsafe { new_header_ptr.as_mut() };

                // Adjust allocated block header
                header.set_block_end(new_block_start);

                // Push new block onto free list
                self.push_onto_free_list(new_header);

                // If possible, coalesce the new block with the next block
                if new_header.block_end() < self.heap_end {
                    let next_header_ptr = new_header.block_end() as *mut BlockHeader;
                    let next_header = unsafe { &mut *next_header_ptr };
                    if !next_header.is_allocated() {
                        self.coalesce_blocks(new_header, next_header);
                    }
                }

                //writeln!(stdio::stdout(), "   new block: {:?}: {:?} + {:?} + {:?} of size {}", new_block_start as *mut u8, new_header, new_header.additional_header(), new_header.footer(), new_header.block_end() - new_header.block_start()).unwrap();
            }

            //writeln!(stdio::stdout(), "   allocated block: {:?}: {:?} of size {}", prev_header_ptr, header, header.block_end() - header.block_start()).unwrap();
            //writeln!(stdio::stdout(), "   returning ptr {:?}, free head: {:?}", payload_start as *mut u8, self.free_head).unwrap();

            return payload_start as _;
        }
    }

    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        //writeln!(stdio::stdout(), " < dealloc with ptr: 0x{:08x}, layout: {:?}", ptr as usize, layout).unwrap();

        let header_pointer_required = layout.align() > mem::align_of::<BlockHeader>();
        //writeln!(stdio::stdout(), "   header pointer required: {}", header_pointer_required).unwrap();

        let mut header_ptr = if header_pointer_required {
            let header_pointer = (ptr as usize - mem::size_of::<NonNull<BlockHeader>>())
                as *const NonNull<BlockHeader>;
            //writeln!(stdio::stdout(), "   header pointer: {:?}", header_pointer).unwrap();
            unsafe { ptr::read(header_pointer) }
        } else {
            unsafe {
                NonNull::new_unchecked((ptr as usize - mem::size_of::<BlockHeader>()) as *mut _)
            }
        };
        let mut header = unsafe { header_ptr.as_mut() };

        //writeln!(stdio::stdout(), "   deallocating block: {:?}: {:?} of size {}", header_ptr, header, header.block_end() - header.block_start()).unwrap();

        // TODO: Consider enumerating the 4 coalesce cases and performing them explicitly rather than chaining simpler operations (which is probably more expensive, even if it may be clearer)

        // Clear allocated flag(s)
        header.clear_allocated();
        let next_block_start = header.block_end();
        if next_block_start != self.heap_end {
            let mut next_block_ptr =
                unsafe { NonNull::new_unchecked(next_block_start as *mut BlockHeader) };
            let next_header = unsafe { next_block_ptr.as_mut() };
            next_header.clear_prev_allocated();
        }
        initialize_footer(header_ptr, header.block_end());

        self.push_onto_free_list(header);

        if header.block_start() != self.heap_start && !header.is_prev_allocated() {
            let mut prev_footer_ptr = unsafe {
                NonNull::new_unchecked(
                    (header.block_start() - mem::size_of::<FreeBlockFooter>())
                        as *mut FreeBlockFooter,
                )
            };
            let prev_footer = unsafe { prev_footer_ptr.as_mut() };
            let prev_header = unsafe { prev_footer.header.as_mut() };
            self.coalesce_blocks(prev_header, header);
            header = prev_header;
        }

        if next_block_start != self.heap_end {
            let next_block_ptr = next_block_start as *mut BlockHeader;
            let next_header = unsafe { &mut *next_block_ptr };
            if !next_header.is_allocated() {
                self.coalesce_blocks(header, next_header);
            }
        }

        //writeln!(stdio::stdout(), "   deallocated block: {:?}: {:?} + {:?} + {:?} of size {}", header as *mut _, header, header.additional_header(), header.footer(), header.block_end() - header.block_start()).unwrap();
        //writeln!(stdio::stdout(), "   free head: {:?}", self.free_head).unwrap();
    }
}

fn initialize_free_block(
    block_start: usize,
    block_end: usize,
    is_prev_allocated: bool,
) -> NonNull<BlockHeader> {
    let header_ptr = block_start as _;
    let header = BlockHeader::new(block_end, is_prev_allocated);
    unsafe {
        ptr::write(header_ptr, header);
    }

    let additional_header_ptr = (block_start + mem::size_of::<BlockHeader>()) as _;
    let additional_header = FreeBlockAdditionalHeader {
        free_prev: None,
        free_next: None,
    };
    unsafe {
        ptr::write(additional_header_ptr, additional_header);
    }

    let ret = unsafe { NonNull::new_unchecked(header_ptr) };

    initialize_footer(ret, block_end);

    ret
}

fn initialize_footer(header: NonNull<BlockHeader>, block_end: usize) {
    let footer_ptr = (block_end - mem::size_of::<FreeBlockFooter>()) as _;
    let footer = FreeBlockFooter { header };
    unsafe {
        ptr::write(footer_ptr, footer);
    }
}

// Assumes `align` is a power of 2
fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

// Assumes `align` is a power of 2
fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
