pub use openseal_sdk_macros::module;
#[doc(hidden)]
pub use mlua::ffi as __ffi;
pub use __ffi::lua_State;

pub mod closure;
pub mod convert;
pub mod metatable;
pub mod registry;
pub mod safety;
pub mod types;
pub mod userdata_methods;

pub use convert::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti};
pub use types::{
    AnyUserData, Error, Function, Lua, OwnedFunction, OwnedTable, Result, Table, TableIter, Value,
};
pub use userdata_methods::{MetaMethod, UserDataMethods};

pub const ABI_VERSION: u32 = 1;

pub mod prelude {
    pub use crate::convert::{FromLua, IntoLua};
    pub use crate::{
        AnyUserData, Error, Function, Lua, MetaMethod, OwnedFunction, OwnedTable, Result, Table,
        TableIter, UserDataMethods,
        Value, module,
    };
}