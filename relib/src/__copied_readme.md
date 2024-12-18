# relib

`relib` is a framework for reloadable dynamic libraries written in Rust.

[![demo](https://github.com/user-attachments/assets/44c87053-8aa1-462f-929f-2a355328387c)](https://github.com/user-attachments/assets/e2da4817-237a-4e90-9c5e-b6f24e4ad57c)

> **note:** currently Linux has the best support, Windows is supported partially, see [support matrix](https://docs.rs/relib/latest/relib/docs/index.html#feature-support-matrix).

## Overview

`relib` is an attempt to make *native* Rust more usable to reload code on the fly, without closing down application, mainly for development (live/hot reload), although `relib` also provides [imports/exports](https://docs.rs/relib/latest/relib/docs/index.html#communication-between-host-and-module) mechanism, which can be used in production.

## Examples

See [examples](https://github.com/xxshady/relib/tree/main/examples/README.md).

## Docs

See [`docs`](https://docs.rs/relib/latest/relib/docs/index.html) of `relib` crate.

## Limitations

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
