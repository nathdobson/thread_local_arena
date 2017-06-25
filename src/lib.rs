#![allow(unused,unused_features)]
#![feature(alloc,optin_builtin_traits,allocator_api,test,unique,heap_api)]
extern crate alloc;
extern crate core;
#[cfg(test)]
extern crate test;
mod owned_arena;
mod arena;
#[cfg(test)]
mod tests;