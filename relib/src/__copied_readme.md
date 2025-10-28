# relib

`relib` is a framework for reloadable dynamic libraries written in Rust.

[![demo](https://github.com/user-attachments/assets/44c87053-8aa1-462f-929f-2a355328387c)](https://github.com/user-attachments/assets/e2da4817-237a-4e90-9c5e-b6f24e4ad57c)

## Platforms supported

Linux and Windows are fully supported, macOS is not supported (not tested), see [support matrix](https://docs.rs/relib/latest/relib/docs/index.html#feature-support-matrix).

## Overview

`relib` tries to be a safe (as much as possible) runtime of native, almost normal Rust programs. Programs that can be safely unloaded (without memory leaks and crashes) without closing the whole OS process.

Since it's not possible to make this completely safe: memory leaks, UB can still happen (for example, due to some unsafe call to C library), you should only use unloading for development (see [live reload](https://github.com/xxshady/relib/tree/main/examples/README.md#live-reload) and [hot reload](https://github.com/xxshady/relib/tree/main/examples/README.md#hot-reload) examples). `relib` can also be used without unloading, see ["Usage without unloading"](https://docs.rs/relib/latest/relib/docs/index.html#usage-without-unloading).

See [feature support matrix](https://docs.rs/relib/latest/relib/docs/index.html#feature-support-matrix) for what `relib` offers to improve unloading of dynamic libraries in Rust. And for what not, check out [caveats](#caveats).

## Examples

See [examples](https://github.com/xxshady/relib/tree/main/examples/README.md).

## Docs

See [`docs`](https://docs.rs/relib/latest/relib/docs/index.html) of `relib` crate.

## Caveats

### Imports/exports runtime validation

*Currently*, `relib` doesn't check at runtime that function signatures (arguments, return types) specified in imports and exports traits (`main` and [`before_unload`](https://docs.rs/relib/latest/relib/docs/index.html#before_unload) as well) are exactly the same for host and module. But it can be easily solved on your side, see `live_reload_extended` [example](https://github.com/xxshady/relib/tree/main/examples/README.md#live-reload-extended).

### ABI stability

> [Why would I want a stable ABI? And what even is an ABI?](https://docs.rs/stabby/latest/stabby/#why-would-i-want-a-stable-abi-and-what-even-is-an-abi)

To ensure at least something about ABI `relib` **checks and requires that host and module are compiled with the same rustc and `relib` version**.

For ABI-stable types, you can use abi_stable or stabby crate for it, see `abi_stable` usage [example](https://github.com/xxshady/relib/tree/main/examples/README.md#usage-with-abi_stable-crate).

### File descriptors and network sockets

*Currently*, `relib` knows nothing about file descriptors or network sockets (unlike [background threads](https://docs.rs/relib/latest/relib/docs/index.html#background-threads-check)) so, for example, if your program stores them in static items and does not properly close them they will leak after unloading.

**note:** relib provides [`before_unload`](https://docs.rs/relib/latest/relib/docs/index.html#before_unload) callback API when you need to cleanup something manually (similar to Rust Drop).

### Dead locks

If your program (module) deadlocks unloading won't work and you will have to kill the whole process.

### Moving non-`Copy` types between host and module

#### Return values

Non-`Copy` types (for example, a heap allocated string) are always implicitly cloned (and must implement `Clone` trait) on host-module boundary when returned from an export or import. Since host and module can use different global [allocators](https://doc.rust-lang.org/stable/std/alloc/index.html) and [`dealloc`](https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html#tymethod.dealloc) expects a pointer allocated exactly via this global allocator.

For example:
```rust
// a type that is common for host and module
#[repr(C)]
#[derive(Debug)]
struct MemoryChunk {
  ptr: *const u8,
  len: usize,
}

// allocates new chunk of memory using global allocator (will be called in generated bindings)
impl Clone for MemoryChunk { ... }

// deallocates it (will be called in generated bindings)
impl Drop for MemoryChunk { ... }

// host:
impl Imports for ModuleImportsImpl {
  fn example() -> MemoryChunk {
    MemoryChunk { ... }
  }
}

// module:
// returned value will be implicitly cloned by using Clone trait
let chunk: MemoryChunk = unsafe { gen_imports::example() }; // gen_imports is defined by relib_interface::include_imports!()
```

**note:** it's still possible to use raw pointers to avoid cloning if you're sure of what you're doing.

#### Returning shallow-clone types

In order to safely move a type between host and module we also need to move it's data because everything will be gone after unloading. With deep-clone types like `Vec<T>` it's simple: we just clone the vector with it's data and now we can do anything with it. But in case with `&'static str` Clone trait does not clone the data. Same with `Rc<T>` and other types that are reference-counting pointers.

**note:** imagine that `Vec<T>`, `&'static str`, etc. are ABI-stable for these examples.

**note:** you can still return shallow-clone types if their lifetime is linked to the input lifetime (see [Lifetime bounds in imports and exports](#lifetime-bounds-in-imports-and-exports)).

#### Parameters

Parameters are limited to `Copy` types, moving non-`Copy` types is not possible.

For example:
```rust
// a type that is common for host and module
#[repr(C)]
#[derive(Debug)]
struct MemoryChunk {
  ptr: *const u8,
  len: usize,
}

// allocates new chunk of memory using global allocator
impl Clone for MemoryChunk { ... }

// deallocates it
impl Drop for MemoryChunk { ... }

// host:
impl Imports for ModuleImportsImpl {
  fn example(chunk: &MemoryChunk) {
    // if owned value is needed just call .to_owned() explicitly:
    let chunk = chunk.to_owned();
  }
}

// module:
let chunk = MemoryChunk { ... };
unsafe { gen_imports::example(&chunk) }; // gen_imports is defined by relib_interface::include_imports!()
```

##### Why parameters are limited to `Copy` types?

Because when you can you should pass values by reference to avoid cost of the cloning allocations.

It is the same reason as with return values: host and module can use different global [allocators](https://doc.rust-lang.org/stable/std/alloc/index.html) and [`dealloc`](https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html#tymethod.dealloc) expects a pointer allocated exactly via this global allocator. So if moving non-`Copy` types would be possible `relib` would still clone parameters implicitly.

### Lifetime elision in imports and exports

Due to the code generation this code may not compile: (`RStr` is ABI-stable equivalent of `&str` from [abi_stable](https://docs.rs/abi_stable/latest/abi_stable/std_types/struct.RStr.html))

```rust
// shared:
pub trait Exports {
  fn ret_ref(str: RStr) -> RStr;
}

// module:
impl Exports for ModuleExportsImpl {
  fn ret_ref(str: RStr) -> RStr {
    str.slice(..) // equal to str[..]
  }
}
```

Will result in:
```txt
error[E0621]: explicit lifetime required in the type of `str`
   --> .../generated_module_exports.rs:234:9
```

In order to fix it you need add explicit lifetime to trait

```rust
pub trait Exports {
  fn ret_ref<'a>(str: RStr<'a>) -> RStr<'a>;
}
```

### Lifetime bounds in imports and exports

It's not possible specify lifetime bounds for imports and exports as it's too complex to implement (there is no `for<'a, 'b: 'a> fn(...)` syntax).

Example: (`RStr` is ABI-stable equivalent of `&str` from [abi_stable](https://docs.rs/abi_stable/latest/abi_stable/std_types/struct.RStr.html))

```rust
pub trait Exports {
  fn returns_b<'a, 'b: 'a>(a: RStr<'a>, b: RStr<'b>) -> RStr<'a> {
    b
  }
}
```

### Each module has its own standard library

Each compiled module (.dll or .so) uses its own copy of standard library. Because of this, for example, [`std::thread::current().id()`](https://doc.rust-lang.org/stable/std/thread/fn.current.html) called in a module may return different id compared to the host, since each module has it's own thread id counter (in this particular case you can use [thread_id](https://docs.rs/thread-id) instead to get thread identifiers from operating system).

## Why dynamic libraries when we already have WASM?

If you can you should use WebAssembly since it's much more memory-safe approach. But what if WASM is not enough for you for some of these reasons: (some of which may be resolved in the future)

- you need to communicate with C++ or C
- you want to use all features of Rust (for example, multi threading, panics, backtraces may not be supported really well in WASM ecosystem)
- you've already written something in normal Rust and don't want to rewrite it to work in WASM
- you don't need sandboxing/isolation
- performance
- bugs in WASM runtimes

## Resources that helped me create this tool

Awesome fasterthanlime's article ❤️ <https://fasterthanli.me/articles/so-you-want-to-live-reload-rust>
