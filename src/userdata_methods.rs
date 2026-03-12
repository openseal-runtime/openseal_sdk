use crate::convert::SelfArgs;
use crate::types::{Function, Lua, Result};
use std::marker::PhantomData;

pub enum MetaMethod {
    Index,
    NewIndex,
    Len,
    ToString,
    Call,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Le,
}

impl MetaMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetaMethod::Index => "__index",
            MetaMethod::NewIndex => "__newindex",
            MetaMethod::Len => "__len",
            MetaMethod::ToString => "__tostring",
            MetaMethod::Call => "__call",
            MetaMethod::Add => "__add",
            MetaMethod::Sub => "__sub",
            MetaMethod::Mul => "__mul",
            MetaMethod::Div => "__div",
            MetaMethod::Eq => "__eq",
            MetaMethod::Lt => "__lt",
            MetaMethod::Le => "__le",
        }
    }
}

pub struct UserDataMethods<'lua, T: 'static> {
    lua: &'lua Lua,
    _marker: PhantomData<T>,
}

impl<'lua, T: 'static> UserDataMethods<'lua, T> {
    pub(crate) fn new(lua: &'lua Lua) -> Self {
        Self {
            lua,
            _marker: PhantomData,
        }
    }

    pub fn add_method<A, R, F>(&self, name: &str, func: F) -> Result<()>
    where
        A: crate::convert::FromLuaMulti + 'static,
        R: crate::convert::IntoLuaMulti + 'static,
        F: Fn(&Lua, &T, A) -> Result<R> + 'static,
    {
        let callback = self.lua.create_function(move |lua, args: SelfArgs<A>| {
            args.0.with_ref::<T, Result<R>>(|data| func(lua, data, args.1))?
        })?;

        self.lua.register_userdata_method::<T>(name, callback)
    }

    pub fn add_method_mut<A, R, F>(&self, name: &str, func: F) -> Result<()>
    where
        A: crate::convert::FromLuaMulti + 'static,
        R: crate::convert::IntoLuaMulti + 'static,
        F: Fn(&Lua, &mut T, A) -> Result<R> + 'static,
    {
        let callback = self.lua.create_function(move |lua, args: SelfArgs<A>| {
            args.0.with_mut::<T, Result<R>>(|data| func(lua, data, args.1))?
        })?;

        self.lua.register_userdata_method::<T>(name, callback)
    }

    pub fn add_meta_method<A, R, F>(&self, metamethod: MetaMethod, func: F) -> Result<()>
    where
        A: crate::convert::FromLuaMulti + 'static,
        R: crate::convert::IntoLuaMulti + 'static,
        F: Fn(&Lua, &T, A) -> Result<R> + 'static,
    {
        self.add_method(metamethod.as_str(), func)
    }

    pub fn add_raw_meta_function(&self, metamethod: MetaMethod, callback: Function) -> Result<()> {
        self.lua
            .register_userdata_method::<T>(metamethod.as_str(), callback)
    }
}