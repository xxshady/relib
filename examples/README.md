# `relib` examples

## [Basic template](https://github.com/xxshady/relib-template)

How to run:<br>
`cargo build --workspace`<br>
`cargo run`

## [Usage with `abi_stable` crate](./abi_stable_usage)

How to run:<br>
`cargo build --workspace`<br>
`cargo run`

## [Custom global allocator](./custom_global_alloc)

How to run:<br>
`cargo build` (in examples/custom_global_alloc directory)<br>
`cargo run --bin basic_host <path>` (in repo root)

> replace `<path>` with `examples/custom_global_alloc/target/debug/libcustom_global_alloc.so` on linux and `examples/custom_global_alloc/target/debug/custom_global_alloc.dll` on windows
