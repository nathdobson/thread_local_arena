scoped_arena
============
A fast and easy-to-use thread local arena allocator. Each standard
heap-allocating type like Box and Vec should have a corresponding arena version
like ArenaBox and ArenaVec. The Arena versions are typically faster for smaller
allocations. The downside is that they don't implement Send. These custom types
should eventually be replaced with an allocator parameter to the standard types.

Q: Is it safe?
A: Probably. There are two basic assumptions for safety: 
