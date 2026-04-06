# relib

`relib` is a framework for reloadable dynamic libraries written in Rust.

[![demo](https://github.com/user-attachments/assets/44c87053-8aa1-462f-929f-2a355328387c)](https://github.com/user-attachments/assets/e2da4817-237a-4e90-9c5e-b6f24e4ad57c)

## Platforms supported

Linux and Windows are fully supported. macOS is currently unsupported (untested); see the [support matrix](https://docs.rs/relib/latest/relib/docs/index.html#feature-support-matrix) for details.

## Overview

`relib` aims to provide a safe (as much as possible) runtime for native, nearly standard Rust programs. It allows programs to be unloaded without memory leaks or crashes and without terminating the entire OS process.

Because total safety is impossible to guarantee - memory leaks and undefined behavior (UB) can still occur (e.g., through unsafe C library calls) - you should generally only use unloading during development (see the [live reload](https://github.com/xxshady/relib/tree/main/examples/README.md#live-reload) and [hot reload](https://github.com/xxshady/relib/tree/main/examples/README.md#hot-reload) examples). `relib` can also be used without unloading; see ["Usage without unloading"](https://docs.rs/relib/latest/relib/docs/index.html#usage-without-unloading).

Check the [feature support matrix](https://docs.rs/relib/latest/relib/docs/index.html#feature-support-matrix) to see how `relib` improves the unloading of dynamic libraries in Rust, and see [caveats](#caveats) for known limitations.

## Examples

See the [examples](https://github.com/xxshady/relib/tree/main/examples/README.md) directory.

## Docs

See [`docs`](https://docs.rs/relib/latest/relib/docs/index.html) of `relib` crate.

## Caveats

### Imports/exports runtime validation

Currently, `relib` does not perform runtime checks to ensure that function signatures (arguments and return types) specified in import and export traits (including `main` and [`before_unload`](https://docs.rs/relib/latest/relib/docs/index.html#before_unload)) match exactly between the host and the module. However, this can be handled manually; see the `live_reload_extended` [example](https://github.com/xxshady/relib/tree/main/examples/README.md#live-reload-extended).

### ABI stability

> [Why would I want a stable ABI? And what even is an ABI?](https://docs.rs/stabby/latest/stabby/#why-would-i-want-a-stable-abi-and-what-even-is-an-abi)

To ensure basic ABI compatibility, **`relib` requires that both the host and the module be compiled with the same version of `rustc` and `relib`**.

For ABI-stable types, you can use the `abi_stable` or `stabby` crates. See the `abi_stable` [usage example](https://github.com/xxshady/relib/tree/main/examples/README.md#usage-with-abi_stable-crate).

### File descriptors and network sockets

_Currently_, `relib` does not track file descriptors or network sockets (unlike [background threads](https://docs.rs/relib/latest/relib/docs/index.html#background-threads-check)). If your program stores these in static items and doesn't close them, they will leak after unloading.

**note:** `relib` provides a [`before_unload`](https://docs.rs/relib/latest/relib/docs/index.html#before_unload) callback API for manual cleanup, similar to Rust's `Drop` trait.

### Deadlocks

If a module deadlocks, unloading will fail, and you will be forced to terminate the entire process.

### Moving non-`Copy` types between host and module

#### Return values

Non-`Copy` types (e.g., a heap-allocated `String`) are always implicitly cloned when crossing the host-module boundary. Consequently, these types must implement the `Clone` trait. This is necessary because the host and module may use different global [allocators](https://doc.rust-lang.org/stable/std/alloc/index.html), and [`dealloc`](https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html#tymethod.dealloc) requires a pointer to be freed by the same allocator that created it.

**Example:**

```rust
// A type common to both host and module
#[repr(C)]
#[derive(Debug)]
struct MemoryChunk {
  ptr: *const u8,
  len: usize,
}

// Allocates a new chunk using the global allocator (will be called in generated bindings)
impl Clone for MemoryChunk { ... }

// Deallocates the chunk
impl Drop for MemoryChunk { ... }

// host:
impl Imports for ModuleImportsImpl {
  fn example() -> MemoryChunk {
    MemoryChunk { ... }
  }
}

// module:
// The returned value is implicitly cloned using the Clone trait
let chunk: MemoryChunk = unsafe { gen_imports::example() };
```

**note:** you can still use raw pointers to avoid cloning if you're sure of what you're doing.

#### Returning shallow-clone types

To safely move a type between host and module, we must move its underlying data, as the original memory space is invalidated after module unloading. While deep-clone types like `Vec<T>` are straightforward, types like `&'static str` or `Rc<T>` do not clone the underlying data.

**note:** imagine that `Vec<T>`, `&'static str`, etc. are ABI-stable for these examples.

**note:** you can still return shallow-clone types if their lifetime is linked to the input lifetime (see [Lifetime bounds in imports and exports](#lifetime-bounds-in-imports-and-exports)).

#### Parameters

Parameters are limited to `Copy` types; moving owned non-`Copy` types as parameters is not supported.

**Example:**

```rust
// host:
impl Imports for ModuleImportsImpl {
  fn example(chunk: &MemoryChunk) {
    // If an owned value is needed, call .to_owned() explicitly:
    let chunk = chunk.to_owned();
  }
}

// module:
let chunk = MemoryChunk { ... };
unsafe { gen_imports::example(&chunk) };
```

##### Why are parameters limited to `Copy` types?

This encourages passing values by reference to avoid the cost of unnecessary allocations. Because the host and module may use different allocators, `relib` would have to implicitly clone non-`Copy` parameters anyway to ensure safe deallocation.

### Lifetime Elision in imports and exports

Due to the way code is generated, the following may not compile (where `RStr` is an ABI-stable equivalent of `&str`):

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

To fix this you need add explicit lifetime to trait:

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

Each compiled module (.dll or .so) uses its own instance of the Rust standard library. As a result, certain globals may behave unexpectedly. For example, [`std::thread::current().id()`](https://doc.rust-lang.org/stable/std/thread/fn.current.html) called inside a module might return a different ID than in the host, as each instance has its own thread ID counter. In this particular case you can use a crate like [thread_id](https://docs.rs/thread-id) to retrieve identifiers directly from the OS.

<br>

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
