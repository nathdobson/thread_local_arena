use std::cell::UnsafeCell;
use alloc::raw_vec::RawVec;
use std::mem;
use std::cmp;
use std::ptr::null_mut;
use std::isize;
use alloc::allocator::Alloc;
use alloc::allocator::Layout;
use alloc::allocator::AllocErr;
use alloc::allocator::Excess;
use alloc::allocator::CannotReallocInPlace;
use std::ptr;

const INITIAL_BLOCK_CAPACITY: usize = 4096;
const BLOCK_ALIGNMENT: usize = 1;
const LARGEST_POWER_OF_TWO: usize = 1 + (isize::MAX as usize);
pub struct OwnedArenaBlock {
    vec: RawVec<u8>,
    len: usize,
}
fn align_padding(value: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
    let result = (alignment - (value & (alignment - 1))) & (alignment - 1);
    debug_assert!(result < alignment);
    debug_assert!(result < LARGEST_POWER_OF_TWO);
    result
}
fn is_aligned(value: usize, alignment: usize) -> bool {
    (value & (alignment - 1)) == 0
}
fn check_layout(layout: Layout) -> Result<(), AllocErr> {
    if layout.size() > LARGEST_POWER_OF_TWO {
        return Err(AllocErr::Unsupported { details: "Bigger than largest power of two" });
    }
    debug_assert!(layout.size() > 0);
    Ok(())
}
fn debug_check_layout(layout: Layout) {
    debug_assert!(layout.size() <= LARGEST_POWER_OF_TWO);
    debug_assert!(layout.size() > 0);
}
impl OwnedArenaBlock {
    pub fn with_capacity(size: usize) -> Result<Self, AllocErr> {
        Ok(OwnedArenaBlock {
            //TODO: propagate failure here
            vec: RawVec::with_capacity(size),
            len: 0,
        })
    }
    unsafe fn is_head(&self, ptr: *mut u8, layout: Layout) -> bool {
        ptr.offset(layout.size() as isize) == self.vec.ptr().offset(self.len as isize)
    }
    unsafe fn reserve(&mut self, increment: usize, request: Layout) -> Result<(), AllocErr> {
        if self.vec.cap() - self.len >= increment ||
           self.vec.reserve_in_place(self.len, increment) {
            self.len += increment;
            Ok(())
        } else {
            Err(AllocErr::Exhausted { request: request })
        }
    }
}
unsafe impl Alloc for OwnedArenaBlock {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        check_layout(layout.clone());
        let padding = align_padding(self.vec.ptr() as usize + self.len, layout.align());
        debug_assert!(padding < LARGEST_POWER_OF_TWO);
        let increment = layout.size() + padding;
        let offset = self.len + padding;
        self.reserve(increment, layout)?;
        Ok(self.vec.ptr().offset(offset as isize))
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        debug_check_layout(layout.clone());
    }
    unsafe fn realloc(&mut self,
                      ptr: *mut u8,
                      old_layout: Layout,
                      new_layout: Layout)
                      -> Result<*mut u8, AllocErr> {
        debug_check_layout(old_layout.clone());
        check_layout(new_layout.clone());
        if self.is_head(ptr, old_layout.clone()) {
            self.len -= old_layout.size();
            match self.alloc(new_layout) {
                Ok(new_ptr) => {
                    if new_ptr != ptr {
                        ptr::copy(ptr, new_ptr, old_layout.size());
                    }
                    Ok(new_ptr)
                }
                Err(err) => {
                    self.len += old_layout.size();
                    Err(err)
                }
            }
        } else {
            let new_ptr = self.alloc(new_layout)?;
            ptr::copy_nonoverlapping(ptr, new_ptr, old_layout.size());
            Ok(new_ptr)
        }
    }
    unsafe fn grow_in_place(&mut self,
                            ptr: *mut u8,
                            old_layout: Layout,
                            new_layout: Layout)
                            -> Result<(), CannotReallocInPlace> {
        debug_check_layout(old_layout.clone());
        check_layout(new_layout.clone());
        if self.is_head(ptr, old_layout.clone()) {
            if is_aligned(ptr as usize, new_layout.align()) {
                self.reserve(new_layout.size() - old_layout.size(), new_layout)
                    .map_err(|_| CannotReallocInPlace)
            } else {
                Err(CannotReallocInPlace)
            }
        } else {
            Err(CannotReallocInPlace)
        }
    }
    unsafe fn shrink_in_place(&mut self,
                              ptr: *mut u8,
                              old_layout: Layout,
                              new_layout: Layout)
                              -> Result<(), CannotReallocInPlace> {
        debug_check_layout(old_layout.clone());
        check_layout(new_layout.clone());
        if self.is_head(ptr, old_layout.clone()) {
            if is_aligned(ptr as usize, new_layout.align()) {
                self.len -= old_layout.size();
                self.len += new_layout.size();
                Ok(())
            } else {
                Err(CannotReallocInPlace)
            }
        } else {
            Err(CannotReallocInPlace)
        }
    }
}
pub struct OwnedArena {
    blocks: Vec<OwnedArenaBlock>,
}
impl OwnedArena {
    pub fn new() -> Result<Self, AllocErr> {
        Ok(OwnedArena { blocks: vec![OwnedArenaBlock::with_capacity(INITIAL_BLOCK_CAPACITY)?] })
    }
    fn last_mut(&mut self) -> &mut OwnedArenaBlock {
        self.blocks.last_mut().unwrap()
    }
    unsafe fn new_block(&mut self, layout: Layout) -> Result<&mut OwnedArenaBlock, AllocErr> {
        let new_capacity = cmp::max(self.blocks.last().unwrap().vec.cap() * 2,
                                    layout.size() + layout.align());
        self.blocks
            .push(OwnedArenaBlock::with_capacity(new_capacity)?);
        Ok(self.last_mut())
    }
    pub unsafe fn arena_scoped<F, T>(&mut self, callback: F) -> T
        where F: FnOnce() -> T,
              F: Send,
              T: Send
    {
        let old_block_count = self.blocks.len();
        let old_len = self.blocks.last().unwrap().len;
        let result = callback();
        self.blocks[old_len - 1].len = old_len;
        // If we reused all the new blocks, we would pay some cpu and
        // fragmentation cost because of transitions between available blocks.
        // If we deallocated all the new blocks, we might pay a high cost to
        // constantly allocate and deallocate from malloc. The compromise is
        // to keep the largest block.
        // TODO: eventually return some memory to malloc if the largest block
        // is too much larger than what is needed.
        let largest_new_block = self.blocks.drain(old_len..).last();
        if let Some(mut largest_new_block) = largest_new_block {
            largest_new_block.len = 0;
            self.blocks.push(largest_new_block);
        }
        result
    }
}
unsafe impl Alloc for OwnedArena {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        check_layout(layout.clone())?;
        match self.last_mut().alloc(layout.clone()) {
            Ok(result) => Ok(result),
            Err(_) => self.new_block(layout.clone())?.alloc(layout.clone()),
        }
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        // It is possible to implement this for the case when all deallocations
        // have the same alignment and occur in reverse allocation order.
        // However if this is left empty, most destructors optimize to
        // the empty function. The performance improvement and performance
        // predictability of a do-nothing implementation is probably worth it.
    }
    unsafe fn realloc(&mut self,
                      ptr: *mut u8,
                      old_layout: Layout,
                      new_layout: Layout)
                      -> Result<*mut u8, AllocErr> {
        debug_check_layout(old_layout.clone());
        check_layout(new_layout.clone());
        match self.last_mut().realloc(ptr, old_layout.clone(), new_layout.clone()) {
            Ok(result) => Ok(result),
            Err(_) => self.new_block(new_layout.clone())?.alloc(new_layout),
        }
    }
    unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
        self.last_mut().alloc_excess(layout)
    }
    unsafe fn realloc_excess(&mut self,
                             ptr: *mut u8,
                             old_layout: Layout,
                             new_layout: Layout)
                             -> Result<Excess, AllocErr> {
        debug_check_layout(old_layout.clone());
        check_layout(new_layout.clone());
        match self.last_mut().realloc_excess(ptr, old_layout.clone(), new_layout.clone()) {
            Ok(result) => Ok(result),
            Err(_) => self.new_block(new_layout.clone())?.alloc_excess(new_layout),
        }
    }
    unsafe fn grow_in_place(&mut self,
                            ptr: *mut u8,
                            old_layout: Layout,
                            new_layout: Layout)
                            -> Result<(), CannotReallocInPlace> {
        debug_check_layout(old_layout.clone());
        check_layout(new_layout.clone());
        self.last_mut().grow_in_place(ptr, old_layout, new_layout)
    }
    unsafe fn shrink_in_place(&mut self,
                              ptr: *mut u8,
                              old_layout: Layout,
                              new_layout: Layout)
                              -> Result<(), CannotReallocInPlace> {
        debug_check_layout(old_layout.clone());
        check_layout(new_layout.clone());
        self.last_mut().shrink_in_place(ptr, old_layout, new_layout)
    }
}
