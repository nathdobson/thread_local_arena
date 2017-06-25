use std::cell::UnsafeCell;
use owned_arena::OwnedArena;
use alloc::allocator::Alloc;
use alloc::allocator::Layout;
use alloc::allocator::AllocErr;
use alloc::allocator::CannotReallocInPlace;
use alloc::allocator::Excess;
use std::ptr::Unique;
thread_local! {
    static ARENA: UnsafeCell<OwnedArena> = UnsafeCell::new(OwnedArena::new().unwrap());
}
// Invokes the callback. Any memory that the callback arena-allocates will be
// returned to the arena. This function is the only way to return memory to the
// arena (Although you can destroy an arena by terminating the thread). Note
// that whether the arena returns any memory to malloc is not explicitly part of
// the spec.
//
// This function is unsafe because a you could use thread_local
// RefCell<Option<ArenaBox<T>>> to return arena allocated memory from the
// callback and create a dangling pointer. It's actually quite hard to
// accidentally get undefined behavior from this.
pub unsafe fn arena_scoped<F, T>(callback: F) -> T
    where F: FnOnce() -> T,
          F: Send,
          T: Send
{
    ARENA.with(|arena| (*arena.get()).arena_scoped(callback))
}
#[derive(Eq,Ord,PartialEq,PartialOrd,Hash,Debug,Copy,Clone,Default)]
pub struct Arena;
impl !Send for Arena {}
unsafe impl Alloc for Arena {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        ARENA.with(|arena| (*arena.get()).alloc(layout))
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        ARENA.with(|arena| (*arena.get()).dealloc(ptr, layout))
    }
    fn oom(&mut self, e: AllocErr) -> ! {
        unsafe { ARENA.with(|arena| (*arena.get()).oom(e)) }
    }
    fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        unsafe { ARENA.with(|arena| (*arena.get()).usable_size(layout)) }
    }
    unsafe fn realloc(&mut self,
                      ptr: *mut u8,
                      layout: Layout,
                      new_layout: Layout)
                      -> Result<*mut u8, AllocErr> {
        ARENA.with(|arena| (*arena.get()).realloc(ptr, layout, new_layout))
    }
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        ARENA.with(|arena| (*arena.get()).alloc_zeroed(layout))
    }
    unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
        ARENA.with(|arena| (*arena.get()).alloc_excess(layout))
    }
    unsafe fn realloc_excess(&mut self,
                             ptr: *mut u8,
                             layout: Layout,
                             new_layout: Layout)
                             -> Result<Excess, AllocErr> {
        ARENA.with(|arena| (*arena.get()).realloc_excess(ptr, layout, new_layout))
    }
    unsafe fn grow_in_place(&mut self,
                            ptr: *mut u8,
                            layout: Layout,
                            new_layout: Layout)
                            -> Result<(), CannotReallocInPlace> {
        ARENA.with(|arena| (*arena.get()).grow_in_place(ptr, layout, new_layout))
    }
    unsafe fn shrink_in_place(&mut self,
                              ptr: *mut u8,
                              layout: Layout,
                              new_layout: Layout)
                              -> Result<(), CannotReallocInPlace> {
        ARENA.with(|arena| (*arena.get()).shrink_in_place(ptr, layout, new_layout))
    }
    fn alloc_one<T>(&mut self) -> Result<Unique<T>, AllocErr>
        where Self: Sized
    {
        unsafe { ARENA.with(|arena| (*arena.get()).alloc_one()) }
    }
    unsafe fn dealloc_one<T>(&mut self, ptr: Unique<T>)
        where Self: Sized
    {
        ARENA.with(|arena| (*arena.get()).dealloc_one(ptr))
    }
    fn alloc_array<T>(&mut self, n: usize) -> Result<Unique<T>, AllocErr>
        where Self: Sized
    {
        unsafe { ARENA.with(|arena| (*arena.get()).alloc_array(n)) }
    }
    unsafe fn realloc_array<T>(&mut self,
                               ptr: Unique<T>,
                               n_old: usize,
                               n_new: usize)
                               -> Result<Unique<T>, AllocErr>
        where Self: Sized
    {
        ARENA.with(|arena| (*arena.get()).realloc_array(ptr, n_old, n_new))
    }
    unsafe fn dealloc_array<T>(&mut self, ptr: Unique<T>, n: usize) -> Result<(), AllocErr>
        where Self: Sized
    {
        ARENA.with(|arena| (*arena.get()).dealloc_array(ptr, n))
    }
}
