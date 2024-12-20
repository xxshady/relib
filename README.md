# relib

`relib` is a framework for reloadable dynamic libraries written in Rust.

[![demo](https://github.com/user-attachments/assets/44c87053-8aa1-462f-929f-2a355328387c)](https://github.com/user-attachments/assets/e2da4817-237a-4e90-9c5e-b6f24e4ad57c)

## Platforms supported

Currently Linux has the best support, Windows is supported partially, macOS is not supported (not tested), see [support matrix](https://docs.rs/relib/latest/relib/docs/index.html#feature-support-matrix).

## Overview

`relib` tries to be a safe (as much as possible) runtime of native, almost normal Rust programs. Programs that can be safely unloaded (without memory leaks and crashes) without closing the whole OS process.

Since it's not possible to make this completely safe: memory leaks, UB can still happen (for example, due to some unsafe call to C library), you should only use unloading for development (see [live reload](https://github.com/xxshady/relib/tree/main/examples/README.md#live-reload) example). `relib` also provides [imports/exports](https://docs.rs/relib/latest/relib/docs/index.html#communication-between-host-and-module) mechanism which can also be used in production.

See [feature support matrix](https://docs.rs/relib/latest/relib/docs/index.html#feature-support-matrix) for what `relib` offers to improve unloading of dynamic libraries in Rust. And for what not, check out [limitations](#limitations).

## Examples

See [examples](https://github.com/xxshady/relib/tree/main/examples/README.md).

## Docs

See [`docs`](https://docs.rs/relib/latest/relib/docs/index.html) of `relib` crate.

## Limitations

### Imports/exports runtime validation

*Currently*, `relib` doesn't check in runtime that function signatures (arguments, return types) specified in imports and exports traits (`main` and [`before_unload`](https://docs.rs/relib/latest/relib/docs/index.html#before_unload) as well) are exactly the same for host and module.

### ABI stability

> [Why would I want a stable ABI? And what even is an ABI?](https://docs.rs/stabby/latest/stabby/#why-would-i-want-a-stable-abi-and-what-even-is-an-abi)

To ensure at least something about ABI `relib` **checks and requires that host and module are compiled with the same rustc and `relib` version**.

For ABI stable types, you can use abi_stable or stabby crate for it, see `abi_stable` usage [example](https://github.com/xxshady/relib/tree/main/examples/README.md#usage-with-abi_stable-crate).

### File descriptors and network sockets

*Currently*, `relib` knows nothing about file descriptors or network sockets (unlike [background threads](https://docs.rs/relib/latest/relib/docs/index.html#background-threads-check)) so, for example, if your program stores them in static items and does not properly close them they will leak after unloading.

**note:** relib provides [`before_unload`](https://docs.rs/relib/latest/relib/docs/index.html#before_unload) callback API when you need to cleanup something manually (similar to Rust Drop).

### Dead locks

If your program deadlocks unloading won't work and you will have to kill the whole process.

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
