# relib

`relib` is a framework for reloadable dynamic libraries written in Rust.

[![demo](https://github.com/user-attachments/assets/44c87053-8aa1-462f-929f-2a355328387c)](https://github.com/user-attachments/assets/e2da4817-237a-4e90-9c5e-b6f24e4ad57c)

> **note:** currently Linux has the best support, Windows is supported partially, see [support matrix](TODO: link to docs.rs).

## Overview

`relib` is an attempt to make *native* Rust more usable to reload code on the fly, without closing down application, mainly for development (live/hot reload), although `relib` also provides [imports/exports](TODO: add link to docs.rs) mechanism, which can be used in production.

## Examples

See [examples](https://github.com/xxshady/relib/tree/main/examples/README.md).

## Docs

See [`docs`](TODO: link to docs.rs) of `relib` crate

## Limitations

### File handles and network sockets

*Currently*, `relib` knows nothing about file handles or network sockets (unlike background threads) so, for example, if your program stores them in static items and does not properly close them they will leak after unloading.

### Dead locks

If you program dead-locks `relib` won't do anything with it and you have to kill host process.

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
