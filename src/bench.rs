use test::Bencher;
use alloc::allocator::Alloc;
use alloc::raw_vec::RawVec;
use alloc::heap::HeapAlloc;
use arena::Arena;
fn bench_rawvec<A: Alloc + Copy>(a: A) {
    let vec: Vec<_> = (0..1024 * 10).map(|_| RawVec::<usize, A>::with_capacity_in(10000, a)).collect();
}
#[bench]
fn bench_rawvec_scoped_arena(b: &mut Bencher) {
    b.iter(|| bench_rawvec(Arena));
}
#[bench]
fn bench_rawvec_heap(b: &mut Bencher) {
    b.iter(|| bench_rawvec(HeapAlloc));
}
//#[bench]
//fn bench_rawvec_arena_block(b: &mut Bencher) {
//    b.iter(|| bench_rawvec(ArenaBlock::with_capacity(1024 * 1024 * 8).unwrap()));
//}
//#[bench]
//fn bench_rawvec_arena(b: &mut Bencher) {
//    b.iter(|| bench_rawvec(Arena::new().unwrap()));
//}
