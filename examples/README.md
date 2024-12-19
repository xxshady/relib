# `relib` examples

## [Basic template](https://github.com/xxshady/relib-template)

Use `cargo run` to compile host and module and run host binary.

## [Live reload](./live_reload)

Automatically reload module on code changes.

Use `cargo run` in live_reload directory and then change something in live_reload/module/src.

## [Usage with `abi_stable` crate](./abi_stable_usage)

Use `cargo run` to compile host and module and run host binary.

## [Custom global allocator](./custom_global_alloc)

How to run:<br>
`cargo build` (in examples/custom_global_alloc directory)<br>
`cargo run --bin basic_host <path>` (in repo root)

> replace `<path>` with `examples/custom_global_alloc/target/debug/libcustom_global_alloc.so` on linux and `examples/custom_global_alloc/target/debug/custom_global_alloc.dll` on windows
