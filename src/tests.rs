use test::Bencher;
use alloc::allocator::Alloc;
use alloc::raw_vec::RawVec;
use alloc::heap::HeapAlloc;
use arena::Arena;
use std::thread;
use std::collections::HashSet;
fn bench_rawvec<A: Alloc + Copy>(a: A) {
    let vec: Vec<_> =
        (0..1024 * 10).map(|_| RawVec::<usize, A>::with_capacity_in(10000, a)).collect();
}
#[bench]
fn bench_rawvec_scoped_arena(b: &mut Bencher) {
    b.iter(|| bench_rawvec(Arena));
}
#[bench]
fn bench_rawvec_heap(b: &mut Bencher) {
    b.iter(|| bench_rawvec(HeapAlloc));
}
#[test]
fn test_alloc() {
    unsafe {
        thread::spawn(|| {
            let mut addresses = HashSet::<*mut u8>::new();
            for i in 0..1000 {
                let vec = RawVec::<u8, Arena>::with_capacity_in(i, Arena);
                for k in 0..i {
                    let ptr = vec.ptr().offset(k as isize);
                    assert!(addresses.insert(ptr));
                    *ptr = 1;
                }
            }
            let dont_format_me_bro = 1;
        });
    }
}
