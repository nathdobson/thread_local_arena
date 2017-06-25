# thread_local_arena
[![Build Status](https://travis-ci.org/nathdobson/thread_local_arena.svg?branch=master)](https://travis-ci.org/nathdobson/thread_local_arena)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A fast and easy-to-use thread local arena allocator. Each standard
heap-allocating type like Box and Vec should have a corresponding arena version
like ArenaBox and ArenaVec. The Arena versions are typically faster for smaller
allocations. The downside is that they don't implement Send. These custom types
should eventually be replaced with an allocator parameter to the standard types.

## Safety
This crate should be safe under the following assumptions:
 * The internal implementation is correct and properly handles  overflow.
 * The negative impl of Send on the Arena allocator is sufficient to prevent any lifetime issues with the underlying memory.
 * Third party crates do not violate the contract of Send. The contract of Send is that it is safe to send the type between threads. Most of the existing negative implementations of the Send trait exist to prevent concurrent execution of some method over some shared piece of data (e.g. Rc<T>). The contract that such types require is that if a value is sent from thread A to thread B, thread B cannot access the data until thread A has terminated. A third party crate may incorrectly assume that this is the contract of Send. On it's own, such a crate would not allow safe code to create undefined behavior. However, when combined with thread_local_arena, it would be possible for safe code to create undefined behavior. A classic example where a crate might make this mistake is not requiring the return type of a spawned thread to be Send. It is safe to return Rc from a thread at the end of the thread's lifetime, but not an arena allocated object.

## Performance
It is the intent of this crate to eventually provide the fastest useful arena allocation in Rust. The current implementation is not as fast as it could be.
