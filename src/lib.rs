#![allow(unused,unused_features)]
#![feature(alloc,optin_builtin_traits,allocator_api,test,unique,heap_api)]
/*
alloc,heap_api,,unique,exact_size_is_empty,fused,
   unsize,coerce_unsized,nonzero,oom,test,i128_type,specialization,
   collections_range,core_intrinsics,trusted_len,shared,inclusive_range,
   offset_to,dropck_eyepatch,generic_param_attrs,box_syntax,unboxed_closures,
   placement_new_protocol,fn_traits,
   */
extern crate alloc;
extern crate core;
#[cfg(test)]
extern crate test;
mod owned_arena;
mod arena;
#[cfg(test)]
mod bench;