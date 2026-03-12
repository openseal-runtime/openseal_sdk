# OpenSeal SDK (Rust)

OpenSeal SDK is the native Rust SDK for building Luau modules loaded by OpenSeal Runtime via `runtime.loadlib`.

This crate uses Luau C FFI (`mlua::ffi`) and does **not** depend on mlua high-level runtime abstractions.

## Workspace Layout

- `openseal_sdk/` -> main SDK crate
- `openseal_sdk/macros/` -> proc-macro crate (`openseal_sdk_macros`)

The SDK re-exports the module macro:

- `#[module]` from `openseal_sdk_macros`

## Versioning and ABI

`openseal_sdk` exposes:

- `ABI_VERSION: u32 = 1`

The `#[module]` macro emits:

- `OPENSEAL_ABI_VERSION` symbol
- `luaopen_<function_name>` entrypoint

This is the runtime ABI contract between OpenSeal Runtime and your DLL.

## Quick Start

`Cargo.toml` for a native module:

```toml
[package]
name = "my_native_module"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
openseal_sdk = { path = "../openseal_sdk" }
```

Example module:

```rust
use openseal_sdk::prelude::*;

#[module]
fn mymod(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    t.set("sum", lua.create_function(|_, (a, b): (i32, i32)| {
        Ok(a + b)
    })?)?;

    Ok(t)
}
```

Load from Luau:

```luau
local mod = runtime.loadlib("./bin/my_native_module.dll", "mymod")
print(mod.sum(2, 3)) -- 5
```

## Public API Reference

### Re-exports

- `module` (attribute macro)
- `lua_State`
- `FromLua`, `FromLuaMulti`, `IntoLua`, `IntoLuaMulti`
- `Lua`, `Table`, `TableIter`, `Function`, `AnyUserData`
- `OwnedTable`, `OwnedFunction`
- `Value`
- `Error`, `Result`
- `MetaMethod`, `UserDataMethods`

### `Error`

Variants:

- `Runtime(String)`
- `Type { expected: &'static str, got: String }`
- `TypePath { expected: &'static str, got: String, path: String }`
- `Borrow(String)`

Helpers:

- `with_path(path)` -> upgrades `Type` into `TypePath`

### `Lua`

Core methods:

- `unsafe fn from_raw(*mut lua_State) -> Lua`
- `fn state(&self) -> *mut lua_State`
- `fn stack_top(&self) -> Result<i32>`
- `fn stack_dump(&self) -> Result<String>`
- `fn create_table(&self) -> Result<Table>`
- `fn create_function<A, R, F>(&self, func: F) -> Result<Function>`
- `fn create_userdata<T: 'static>(&self, data: T) -> Result<AnyUserData>`
- `fn register_userdata_method<T: 'static>(&self, name: &str, func: Function) -> Result<()>`
- `fn userdata_methods<T: 'static>(&self) -> UserDataMethods<T>`

### `Table`

- `fn state(&self) -> *mut lua_State`
- `fn set(key, value) -> Result<()>`
- `fn get<T>(key) -> Result<T>`
- `fn raw_set(key, value) -> Result<()>`
- `fn raw_get<T>(key) -> Result<T>`
- `fn set_field(&str, value) -> Result<()>`
- `fn get_field<T>(&str) -> Result<T>`
- `fn contains_key(key) -> Result<bool>`
- `fn len() -> Result<usize>`
- `fn clear() -> Result<()>`
- `fn iter() -> Result<TableIter>`
- `fn set_metamethod(name, Function) -> Result<()>`
- `fn set_metatable(&Table) -> Result<()>`
- `fn get_metatable() -> Result<Option<Table>>`
- `fn set_readonly(bool) -> Result<()>`
- `fn push_to_stack()`
- `fn into_owned() -> OwnedTable`

### `TableIter`

Iterator item:

- `(Value, Value)`

### `Function`

- `fn push_to_stack()`
- `fn into_owned() -> OwnedFunction`

### `AnyUserData`

- `fn push_to_stack()`
- `fn with_ref<T, R>(FnOnce(&T) -> R) -> Result<R>`
- `fn with_mut<T, R>(FnOnce(&mut T) -> R) -> Result<R>`
- `fn set_metatable<T: 'static>() -> Result<()>`

### Ownership wrappers

- `OwnedTable(Table)`
- `OwnedFunction(Function)`

Methods:

- `as_table` / `as_function`
- `into_inner`
- `push_to_stack`

### `Value`

Variants:

- `Nil`
- `Boolean(bool)`
- `Number(f64)`
- `String(String)`
- `Table(Table)`
- `Function(Function)`
- `UserData(AnyUserData)`

## Conversion Traits

### `IntoLua`

Implemented for:

- `String`, `&str`, `Cow<'a, str>`
- `i32`, `u32`, `i64`, `u64`, `f64`, `bool`
- `Option<T>` where `T: IntoLua`
- `Vec<T>`, `VecDeque<T>` where `T: IntoLua`
- `HashMap<String, T>`, `BTreeMap<String, T>` where `T: IntoLua`
- `Table`, `&Table`, `OwnedTable`
- `Function`, `&Function`, `OwnedFunction`
- `AnyUserData`, `&AnyUserData`
- `Value`

### `FromLua`

Implemented for:

- `String`, `Cow<'static, str>`
- `i32`, `u32`, `i64`, `u64`, `f64`, `bool`
- `Option<T>` where `T: FromLua`
- `Vec<T>`, `VecDeque<T>` where `T: FromLua`
- `HashMap<String, T>`, `BTreeMap<String, T>` where `T: FromLua`
- `Table`, `Function`, `AnyUserData`, `Value`

### Multi-value traits

- `FromLuaMulti`:
  - `()`
  - `A`
  - tuples `(A, B)` through `(A, B, C, D, E, F, G, H)`
  - `SelfArgs<A>` (userdata self + args tail)
- `IntoLuaMulti`:
  - `()`
  - `A`
  - tuples `(A, B)` through `(A, B, C, D, E, F, G, H)`

## Userdata API (`UserDataMethods`)

`MetaMethod` enum maps to Luau metamethod names:

- `Index -> "__index"`
- `NewIndex -> "__newindex"`
- `Len -> "__len"`
- `ToString -> "__tostring"`
- `Call -> "__call"`
- `Add -> "__add"`
- `Sub -> "__sub"`
- `Mul -> "__mul"`
- `Div -> "__div"`
- `Eq -> "__eq"`
- `Lt -> "__lt"`
- `Le -> "__le"`

`UserDataMethods<T>` methods:

- `add_method(name, |lua, &T, A| -> Result<R>)`
- `add_method_mut(name, |lua, &mut T, A| -> Result<R>)`
- `add_meta_method(MetaMethod, |lua, &T, A| -> Result<R>)`
- `add_raw_meta_function(MetaMethod, Function)`

## Safety Module

`openseal_sdk::safety` exports:

- `ensure_state(*mut lua_State) -> Result<()>`
- `StackGuard` (stack restore on drop)
- `stack_dump(*mut lua_State) -> Result<String>`

## Low-level Public Modules

The crate also exposes low-level modules for advanced use:

- `closure`
- `metatable`
- `registry`

These are public and available, but they are lower-level building blocks.

## Macro Behavior (`#[module]`)

The macro:

1. Keeps your Rust function unchanged.
2. Emits `OPENSEAL_ABI_VERSION` static.
3. Emits `luaopen_<fn_name>` C entrypoint.
4. Catches Rust panic with `catch_unwind`.
5. On success, pushes returned `Table` and returns `1`.
6. On error/panic, pushes message into Lua and raises `lua_error`.

## Examples

See:

- `examples/table_api.rs`
- `examples/userdata_methods.rs`
- `examples/conversions.rs`

## Build and Validate

From workspace root:

```bash
cargo check -p openseal_sdk
cargo check -p openseal_sdk --examples
```

## Notes

- This SDK is designed for OpenSeal Runtime loading model (`runtime.loadlib`).
- `sealpm` is a separate package manager crate and is not part of this SDK API.