# `relib` examples

## [Basic template](https://github.com/xxshady/relib-template)

[How to use](https://github.com/xxshady/relib-template/blob/main/README.md).

## [Live reload](./live_reload)

Automatically reload module on code changes.

Use `cargo run` in live_reload directory and then change something in live_reload/module/src.

## [Live reload extended](./live_reload_extended)

Adds compatibility check between host and module.
When module is reloaded host checks if it's still on the same "snapshot" of shared crate (which contains interface of imports and exports).

**note:** this is not needed if it's known that the shared crate will never be modified between reloads. For example, when shared crate is external dependency.

<details>
<summary>How it works</summary>

---

shared crate defines build id (which is just a timestamp of when the crate was built) and it's used to check if host and module are using the same "snapshot" of the shared crate.

Why `./run.sh`? host and module crates needs to be built with the same `cargo build` command, from the same root directory,
so that build.rs in shared crate is working correctly, also host binary needs to be copied to avoid conflicts with `cargo build`
runner in the host binary.

Instead of timestamp approach we could, for example, hash shared directory + root Cargo.lock + directories of local dependencies
but it would be more complex to implement.

---

</details>

Use `./run.sh` (you need bash for that if you are on Windows) in live_reload_extended directory and then change something in live_reload_extended/module/src or live_reload_extended/shared/src.

## [Hot reload](./hot_reload)

Automatically reload code without resetting state.

Use `./run.sh` (you need bash for that if you are on Windows) in hot_reload directory and then change something in hot_reload/update_module/src/update_module.rs.

### Crate structure

- `host` - the one and the only
- `state` - `State` structure that is preserved between reloads of `update_module`
- `update_module` - this is the crate that is hot-reloaded (leaked allocations are not collected, yet?)
- `main_module` - this crate is live-reloaded (if changed the state will be reset)
- `main_contract` - contains exports of main_module and imports of main_module & update_module
- `update_contract` - contains imports & exports only of update_module
- `perfect_api` - simple example of library with no global/hidden state
- `imperfect_api` - simple real-world example

**note:** this example also includes live reload (reload with reset of the state), see main_module/src/main_module.rs.

## [Usage with `abi_stable` crate](./abi_stable_usage)

Use `cargo run` to compile host, module crates and run host binary.

**note:** you can also run it without `--features unloading` (see ["Usage without unloading"](https://docs.rs/relib/latest/relib/docs/index.html#usage-without-unloading)).

## [Customized `relib_module::export` proc-macro](./export_main_macro)

This example shows how you can make your own export macro for main function which will validate input code.

- Try to change main function of module, add arguments or change return type.
- Try to build the module using `cargo build --workspace` in export_main_macro directory

## [Custom global allocator](./custom_global_alloc)

How to run:<br>
`cargo build --features unloading` (in examples/custom_global_alloc directory)<br>
`cargo run --bin basic_host --features unloading <path>` (in repo root)

> replace `<path>` with `examples/custom_global_alloc/target/debug/libcustom_global_alloc.so` on linux and `examples/custom_global_alloc/target/debug/custom_global_alloc.dll` on windows

**note:** you can also build it without `--features unloading` (see ["Usage without unloading"](https://docs.rs/relib/latest/relib/docs/index.html#usage-without-unloading)).

## [Different interfaces](./different_interfaces)

That's when if you want to load modules with different exports and imports.

How to run:<br>
`cargo build --features unloading`<br>
`cargo run --features unloading`

**note:** you can also build it without `--features unloading` (see ["Usage without unloading"](https://docs.rs/relib/latest/relib/docs/index.html#usage-without-unloading)).
