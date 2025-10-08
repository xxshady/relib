## Terminology

`relib` uses similar to WASM terminology:

- Host: Rust program which controls modules.
- Module: Rust dynamic library, which can import and export functions to host.

> **note:** host can be an executable or dynamic library, but if dynamic library is already a module it can't be a host.

## Getting started

> If you don't want to repeat all these steps, you can use ready-made [template](https://github.com/xxshady/relib-template)

- Create Rust workspace: create empty directory with the following `Cargo.toml` (at the time of writing there is no cargo command to create workspace):<br>
```toml
[workspace]
resolver = "3" # edition "2024" implies resolver "3"

[workspace.package]
version = "0.1.0"
edition = "2024" # or set a later one
```

- Create host crate: (`--vcs none` to not create unneeded git stuff)<br>
`cargo new host --vcs none`

- Create module crate:<br>
`cargo new --lib module --vcs none`

- Configure module crate to compile as dynamic library, add the following to the `module/Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]
```

- Add relib_host dependency to host crate:<br>
`cargo add relib_host --package host`

- Configure "unloading" feature in `host/Cargo.toml` (see also ["Usage without unloading"](#usage-without-unloading))
```toml
[features]
unloading = ["relib_host/unloading"]
```

- Add the following to `main.rs` of host:
```rust
fn main() {
  let path_to_dylib = if cfg!(target_os = "linux") {
    "target/debug/libmodule.so"
  } else {
    "target/debug/module.dll"
  };

  // `()` means empty imports and exports, here module doesn't import or export anything
  let module = unsafe {
    relib_host::load_module::<()>(path_to_dylib, ())
  };
  let module = module.unwrap_or_else(|e| {
    panic!("module loading failed: {e:#}");
  });

  // main function is unsafe to call (as well as any other module export) because these preconditions are not checked by relib:
  // 1. returned value must be actually `R` at runtime, for example you called this function with type bool but module returns i32.
  // 2. type of return value must be FFI-safe.
  // 3. returned value must not be a reference-counting pointer (see caveats on main docs page/README).
  let returned_value: Option<()> = unsafe {
    module.call_main::<()>()
  };

  // if module panics while executing any export it returns None
  // (panic will be printed by module)
  if returned_value.is_none() {
    println!("module panicked");
  }

  // module.unload() is provided when unloading feature of relib_host crate is enabled
  #[cfg(feature = "unloading")]
  {
    println!("unloading feature is enabled, calling module unload");

    module.unload().unwrap_or_else(|e| {
      panic!("module unloading failed: {e:#}");
    });
  }
}
```

- Add relib_module dependency to module crate:<br>
`cargo add relib_module --package module`

- Configure "unloading" feature in `module/Cargo.toml`
```toml
[features]
unloading = ["relib_module/unloading"]
```

- Add the following to `lib.rs` of module:
```rust
#[relib_module::export]
fn main() {
  println!("hello world");
}
```

- And run host `cargo run --features unloading` (it should also build module crate automatically), which will load and execute module

## Communication between host and module

To communicate between host and module `relib` provides convenient API for declaring imports and exports and implementing them using Rust traits.

> (which is heavily inspired by the [WASM Component Model](https://component-model.bytecodealliance.org/))

### Preparations for imports and exports

(make sure you followed "Getting started" guide)

- Add libloading dependency to host crate:<br>
`cargo add libloading --package host`

- Add `relib_interface` dependency with "include" feature to host and module crates:<br>
`cargo add relib_interface --package host --features include`<br>
`cargo add relib_interface --package module --features include`

- Also add it as build-dependency with "build" feature to host and module crates:<br>
`cargo add relib_interface --package host --features build --build`<br>
`cargo add relib_interface --package module --features build --build`

- Create "shared" crate: `cargo new shared --lib --vcs none`

- Add it as dependency to host and module crates:<br>
`cargo add --path ./shared --package host`<br>
`cargo add --path ./shared --package module`

- Add it as build-dependency as well (it's needed for bindings generation in `relib_interface` crate)<br>
`cargo add --path ./shared --package host --build`<br>
`cargo add --path ./shared --package module --build`

- Define modules in shared crate for imports and exports trait:
```rust
// shared/src/lib.rs:
pub mod exports;
pub mod imports;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");

// shared/src/exports.rs:
pub trait Exports {}

// shared/src/imports.rs:
pub trait Imports {}
```

- Create build script in host crate with the following code:
```rust
// host/build.rs
fn main() {
  relib_interface::host::generate(
    shared::EXPORTS,
    "shared::exports::Exports", 
    shared::IMPORTS,
    "shared::imports::Imports",
  );
}
```

- In module crate as well:
```rust
// module/build.rs
fn main() {
  relib_interface::module::generate(
    shared::EXPORTS,
    "shared::exports::Exports",
    shared::IMPORTS,
    "shared::imports::Imports",
  );
}
```

- Include bindings which will be generated by build.rs:
```rust
// in host/src/main.rs and module/src/lib.rs:

// in top level
relib_interface::include_exports!();
relib_interface::include_imports!();

// these macros expand into:
// mod gen_imports {
//   include!(concat!(env!("OUT_DIR"), "/generated_module_imports.rs"));
// }
// mod gen_exports {
//   include!(concat!(env!("OUT_DIR"), "/generated_module_exports.rs"));
// }
```

- Now try to build everything: `cargo build --workspace --features unloading`, it should give you a few warnings

### Module imports

- Now we can add any function we want to exports and imports, let's add an import:
```rust
// in shared/src/imports.rs:
pub trait Imports {
  fn foo() -> u8;
}

// and implement it in host/src/main.rs:

// gen_imports module is defined by relib_interface::include_imports!()
impl shared::imports::Imports for gen_imports::ModuleImportsImpl {
  fn foo() -> u8 {
    10
  }
}
```

- After that we need to modify `load_module` call in the host crate:
```rust
let module = unsafe {
  relib_host::load_module::<()>(
    path_to_dylib,
    gen_imports::init_imports,
  )
};
```

- And now we can call "foo" from module/src/lib.rs:
```rust
// both imports and exports are unsafe to call since these preconditions are not checked by relib:
// 1. types of arguments and return value must be FFI-safe
//    (you can use abi_stable or stabby crate for it, see "abi_stable_usage" example).
// 2. host and module crates must be compiled with same shared crate code.
// 3. returned value must not be a reference-counting pointer (see caveats on main docs page/README).
let value = unsafe { gen_imports::foo() }; // gen_imports is defined by relib_interface::include_imports!()
dbg!(value); // prints "value = 10"
```

### Module exports

Exports work in a similar way to imports.

```rust
// in shared/src/exports.rs:
pub trait Exports {
  fn foo() -> u8;
}

// implement it in module/src/lib.rs:
// gen_exports module is defined by relib_interface::include_exports!()
impl shared::exports::Exports for gen_exports::ModuleExportsImpl {
  fn bar() -> u8 {
    15
  }
}

// in host/src/main.rs:
let module = unsafe {
  relib_host::load_module::<gen_exports::ModuleExports>(
    path_to_dylib,
    gen_imports::init_imports,
  )
};
let module = module.unwrap_or_else(|e| {
  panic!("module loading failed: {e:#}");
});
```

Except one thing, return value:

```rust
// returns None if module export panics
let value: Option<u8> = unsafe { module.exports().bar() };
```

## `before_unload` 

Module can define callback which will be called when it's is unloaded by host (similar to Rust `Drop`).

**note:** it's only needed when unloading is enabled (see also ["Usage without unloading"](#usage-without-unloading)).

```rust
#[cfg(feature = "unloading")]
#[relib_module::export]
fn before_unload() {
  println!("seems like host called module.unload()!");
}
```

## Usage without unloading

When you need to unload modules `relib` provides memory deallocation, background threads check, etc.

But `relib` can also be used without these features. For example, you probably don't want to reload modules in production since it can be dangerous.

Even without unloading `relib` provides some useful features: imports/exports, panic handling in exports, backtraces [support](#backtraces) for multiple modules (dynamic libraries), and some checks in module loading (see [`LoadError`](https://docs.rs/relib_host/latest/relib_host/enum.LoadError.html)).

### How to turn off module unloading

Disable "unloading" feature in relib_host and relib_module crates (no features are enabled by default). If you followed "Getting started" guide or if you use ready-made [template](https://github.com/xxshady/relib-template) you can simply run
`cargo build --workspace` (without `--features unloading`) to build host and module without unloading feature.

## Module alloc tracker

All heap allocations made in the module are tracked and leaked ones are deallocated on module unload (if unloading feature is enabled).
It's done using `#[global_allocator]` so if you want to set your own global allocator you need to disable all features of relib_module crate, enable "unloading_core" and define your allocator using `relib_module::AllocTracker`. See "Custom global allocator" [example](https://github.com/xxshady/relib/tree/main/examples/README.md#custom-global-allocator).

## Feature support matrix

| Feature                                                    | Linux   | Windows                             |
|----------------------------------------------------------- |-------  |------------------------------------  |
| Memory deallocation [(?)](#memory-deallocation)            | ✅      | ✅                                   |
| Panic handling [(?)](#panic-handling)                      | ✅      | ✅                                   |
| Thread-locals                                              | ✅      | ✅                                   |
| Background threads check [(?)](#background-threads-check)  | ✅      | ✅                                   |
| Final unload check [(?)](#final-unload-check)              | ✅      | ✅                                   |
| Before load check [(?)](#before-load-check)                | ✅      | ✅                                   |
| Backtraces [(?)](#backtraces)                              | ✅      | ✅                                   |

### Memory deallocation

Active allocations are freed when module is unloaded by host. For example:

```rust
let string = String::from("leak");
// leaked, but will be deallocated when unloaded by host
std::mem::forget(string);

static mut STRING: String = String::new();

// same, Rust statics do not have destructors
// so it will be deallocated by host
unsafe {
  STRING = String::from("leak");
}
```

**note:** keep in mind that only Rust allocations are deallocated, so if you call some C library which has memory leak it won't be freed on module unload (you can use `valgrind` or `heaptrack` to debug such cases).

### Background threads check

Dynamic library cannot be unloaded safely if background threads spawned by it are still running at the time of unloading, so host checks them and returns [`ThreadsStillRunning`](https://docs.rs/relib_host/latest/relib_host/enum.UnloadError.html#variant.ThreadsStillRunning) error if so.

**note:** module can register [`before_unload`](#before_unload) function to join threads when host triggers module [`unload`](https://docs.rs/relib_host/latest/relib_host/struct.Module.html#method.unload)

### Panic handling

#### Exports

When any export (`main`, `before_unload` and implemented on `gen_exports::ModuleExportsImpl`) of module panics it will return `None` to host and panic message will be printed by the module:

```rust
// host:

let module = unsafe {
  relib::load_module::<ModuleExports>("...")
}?;

let value = module.call_main::<()>();
if value.is_none() {
  // module panicked
}

let value = module.exports().foo();
if value.is_none() {
  // same, module panicked
}
```

**note:** not all panics are handled, see a ["double panic"](https://doc.rust-lang.org/std/ops/trait.Drop.html#panics)

##### Behavior of `before_unload`

Currently, `before_unload` is called when `module.unload()` is called after panic, but this may be changed in the future.

#### Imports

When any import panics (implemented on `gen_exports::ModuleImportsImpl`) it will abort the whole process

```rust
// host:
// gen_imports module is defined by relib_interface::include_imports!()
impl shared::imports::Imports for gen_imports::ModuleImportsImpl {
  fn panics() {
    panic!()
  }
}

// module:
unsafe {
  // will output panic and abort entire process (host) with error
  gen_imports::panics();
}
```

### Final unload check

After host called `library.close()` ([`close`](https://docs.rs/libloading/latest/libloading/struct.Library.html#method.close) from libloading) it will check if library has indeed been unloaded. On Linux it's done via reading `/proc/self/maps`.

### Before load check

Before loading a module host checks if module is already loaded or not, if it's loaded [`ModuleAlreadyLoaded`](https://docs.rs/relib_host/latest/relib_host/enum.LoadError.html#variant.ModuleAlreadyLoaded) error will be returned.

### Backtraces

On Linux there are hooks of libc `mmap64` and `munmap` to unmap leaked memory mappings on module unloading in `std::backtrace` since there is no public API for that.<br>
On Windows there is a `dbghelp.dll` hook, which is initialized in `relib_host::load_module` when it's called for the first time. It adds support for backtraces in multiple modules (even without [unloading](#usage-without-unloading) feature).
