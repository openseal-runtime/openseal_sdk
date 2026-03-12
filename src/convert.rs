use crate::__ffi as ffi;
use crate::safety::StackGuard;
use crate::types::*;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::os::raw::c_int;

pub trait IntoLua {
    fn push_into(self, state: *mut lua_State) -> Result<()>;
}

impl IntoLua for String {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        unsafe { ffi::lua_pushlstring_(s, self.as_ptr() as *const _, self.len()) }
        Ok(())
    }
}

impl<'a> IntoLua for Cow<'a, str> {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        match self {
            Cow::Borrowed(v) => v.push_into(s),
            Cow::Owned(v) => v.push_into(s),
        }
    }
}

impl IntoLua for &str {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        unsafe { ffi::lua_pushlstring_(s, self.as_ptr() as *const _, self.len()) }
        Ok(())
    }
}

impl IntoLua for i32 {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        unsafe { ffi::lua_pushnumber(s, self as f64) }
        Ok(())
    }
}

impl IntoLua for u32 {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        unsafe { ffi::lua_pushnumber(s, self as f64) }
        Ok(())
    }
}

impl IntoLua for i64 {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        unsafe { ffi::lua_pushnumber(s, self as f64) }
        Ok(())
    }
}

impl IntoLua for u64 {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        unsafe { ffi::lua_pushnumber(s, self as f64) }
        Ok(())
    }
}

impl IntoLua for f64 {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        unsafe { ffi::lua_pushnumber(s, self) }
        Ok(())
    }
}

impl IntoLua for bool {
    fn push_into(self, s: *mut lua_State) -> Result<()> {
        unsafe { ffi::lua_pushboolean(s, self as c_int) }
        Ok(())
    }
}

impl<T: IntoLua> IntoLua for Option<T> {
    fn push_into(self, state: *mut lua_State) -> Result<()> {
        match self {
            Some(v) => v.push_into(state),
            None => {
                unsafe { ffi::lua_pushnil(state) }
                Ok(())
            }
        }
    }
}

impl<T: IntoLua> IntoLua for VecDeque<T> {
    fn push_into(self, state: *mut lua_State) -> Result<()> {
        self.into_iter().collect::<Vec<_>>().push_into(state)
    }
}

impl<T: IntoLua> IntoLua for Vec<T> {
    fn push_into(self, state: *mut lua_State) -> Result<()> {
        let table = unsafe {
            ffi::lua_createtable(state, self.len() as c_int, 0);
            Table::from_stack(state)?
        };

        for (i, value) in self.into_iter().enumerate() {
            table.raw_set((i + 1) as i32, value)?;
        }

        table.push_to_stack();
        std::mem::forget(table);
        Ok(())
    }
}

impl<T: IntoLua> IntoLua for BTreeMap<String, T> {
    fn push_into(self, state: *mut lua_State) -> Result<()> {
        let table = unsafe {
            ffi::lua_createtable(state, 0, self.len() as c_int);
            Table::from_stack(state)?
        };

        for (k, v) in self {
            table.raw_set(k, v)?;
        }

        table.push_to_stack();
        std::mem::forget(table);
        Ok(())
    }
}

impl<T: IntoLua> IntoLua for HashMap<String, T> {
    fn push_into(self, state: *mut lua_State) -> Result<()> {
        let table = unsafe {
            ffi::lua_createtable(state, 0, self.len() as c_int);
            Table::from_stack(state)?
        };

        for (k, v) in self {
            table.raw_set(k, v)?;
        }

        table.push_to_stack();
        std::mem::forget(table);
        Ok(())
    }
}

impl IntoLua for OwnedTable {
    fn push_into(self, _state: *mut lua_State) -> Result<()> {
        self.push_to_stack();
        Ok(())
    }
}

impl IntoLua for OwnedFunction {
    fn push_into(self, _state: *mut lua_State) -> Result<()> {
        self.push_to_stack();
        Ok(())
    }
}

impl IntoLua for Table {
    fn push_into(self, _state: *mut lua_State) -> Result<()> {
        self.push_to_stack();
        std::mem::forget(self);
        Ok(())
    }
}

impl IntoLua for &Table {
    fn push_into(self, _state: *mut lua_State) -> Result<()> {
        self.push_to_stack();
        Ok(())
    }
}

impl IntoLua for Function {
    fn push_into(self, _state: *mut lua_State) -> Result<()> {
        self.push_to_stack();
        std::mem::forget(self);
        Ok(())
    }
}

impl IntoLua for &Function {
    fn push_into(self, _state: *mut lua_State) -> Result<()> {
        self.push_to_stack();
        Ok(())
    }
}

impl IntoLua for AnyUserData {
    fn push_into(self, _state: *mut lua_State) -> Result<()> {
        self.push_to_stack();
        std::mem::forget(self);
        Ok(())
    }
}

impl IntoLua for &AnyUserData {
    fn push_into(self, _state: *mut lua_State) -> Result<()> {
        self.push_to_stack();
        Ok(())
    }
}

impl IntoLua for Value {
    fn push_into(self, state: *mut lua_State) -> Result<()> {
        match self {
            Value::Nil => {
                unsafe { ffi::lua_pushnil(state) }
                Ok(())
            }
            Value::Boolean(v) => v.push_into(state),
            Value::Number(v) => v.push_into(state),
            Value::String(v) => v.push_into(state),
            Value::Table(v) => v.push_into(state),
            Value::Function(v) => v.push_into(state),
            Value::UserData(v) => v.push_into(state),
        }
    }
}

pub trait FromLua: Sized {
    fn from_stack(state: *mut lua_State, idx: c_int) -> Result<Self>;
}

impl FromLua for String {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            let mut len = 0;
            let ptr = ffi::lua_tolstring(s, i, &mut len);
            if ptr.is_null() {
                return Err(Error::Type {
                    expected: "string",
                    got: lua_type_name_at(s, i),
                });
            }
            let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }
}

impl FromLua for i32 {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) != ffi::LUA_TNUMBER {
                return Err(Error::Type {
                    expected: "number",
                    got: lua_type_name_at(s, i),
                });
            }
            Ok(ffi::lua_tonumber(s, i) as i32)
        }
    }
}

impl FromLua for u32 {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        i32::from_stack(s, i).map(|v| v as u32)
    }
}

impl FromLua for i64 {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        f64::from_stack(s, i).map(|v| v as i64)
    }
}

impl FromLua for u64 {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        f64::from_stack(s, i).map(|v| v as u64)
    }
}

impl FromLua for f64 {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) != ffi::LUA_TNUMBER {
                return Err(Error::Type {
                    expected: "number",
                    got: lua_type_name_at(s, i),
                });
            }
            Ok(ffi::lua_tonumber(s, i))
        }
    }
}

impl FromLua for bool {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) != ffi::LUA_TBOOLEAN {
                return Err(Error::Type {
                    expected: "boolean",
                    got: lua_type_name_at(s, i),
                });
            }
            Ok(ffi::lua_toboolean(s, i) != 0)
        }
    }
}

impl FromLua for Cow<'static, str> {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        String::from_stack(s, i).map(Cow::Owned)
    }
}

impl<T: FromLua> FromLua for Option<T> {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) == ffi::LUA_TNIL {
                return Ok(None);
            }
        }
        T::from_stack(s, i).map(Some)
    }
}

impl<T: FromLua> FromLua for VecDeque<T> {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        Vec::<T>::from_stack(s, i).map(VecDeque::from)
    }
}

impl<T: FromLua> FromLua for Vec<T> {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) != ffi::LUA_TTABLE {
                return Err(Error::Type {
                    expected: "table",
                    got: lua_type_name_at(s, i),
                });
            }

            let abs = ffi::lua_absindex(s, i);
            let len = ffi::lua_objlen(s, abs) as i32;
            let mut out = Vec::with_capacity(len as usize);
            for idx in 1..=len {
                ffi::lua_rawgeti_(s, abs, idx);
                let value = T::from_stack(s, -1)?;
                ffi::lua_pop(s, 1);
                out.push(value);
            }
            Ok(out)
        }
    }
}

impl<T: FromLua> FromLua for BTreeMap<String, T> {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        HashMap::<String, T>::from_stack(s, i).map(|m| m.into_iter().collect())
    }
}

impl<T: FromLua> FromLua for HashMap<String, T> {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) != ffi::LUA_TTABLE {
                return Err(Error::Type {
                    expected: "table",
                    got: lua_type_name_at(s, i),
                });
            }

            let _guard = StackGuard::new(s)?;
            let abs = ffi::lua_absindex(s, i);
            ffi::lua_pushnil(s);
            let mut out = HashMap::new();

            while ffi::lua_next(s, abs) != 0 {
                let key = String::from_stack(s, -2)?;
                let value = T::from_stack(s, -1)?;
                out.insert(key, value);
                ffi::lua_pop(s, 1);
            }

            Ok(out)
        }
    }
}

impl FromLua for Table {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) != ffi::LUA_TTABLE {
                return Err(Error::Type {
                    expected: "table",
                    got: lua_type_name_at(s, i),
                });
            }
            ffi::lua_pushvalue(s, i);
            Table::from_stack(s)
        }
    }
}

impl FromLua for Function {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) != ffi::LUA_TFUNCTION {
                return Err(Error::Type {
                    expected: "function",
                    got: lua_type_name_at(s, i),
                });
            }
            ffi::lua_pushvalue(s, i);
            Function::from_stack(s)
        }
    }
}

impl FromLua for AnyUserData {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            if ffi::lua_type(s, i) != ffi::LUA_TUSERDATA {
                return Err(Error::Type {
                    expected: "userdata",
                    got: lua_type_name_at(s, i),
                });
            }
            ffi::lua_pushvalue(s, i);
            AnyUserData::from_stack(s)
        }
    }
}

impl FromLua for Value {
    fn from_stack(s: *mut lua_State, i: c_int) -> Result<Self> {
        unsafe {
            match ffi::lua_type(s, i) {
                ffi::LUA_TNIL => Ok(Value::Nil),
                ffi::LUA_TBOOLEAN => Ok(Value::Boolean(ffi::lua_toboolean(s, i) != 0)),
                ffi::LUA_TNUMBER => Ok(Value::Number(ffi::lua_tonumber(s, i))),
                ffi::LUA_TSTRING => Ok(Value::String(String::from_stack(s, i)?)),
                ffi::LUA_TTABLE => { ffi::lua_pushvalue(s, i); Ok(Value::Table(Table::from_stack(s)?)) },
                ffi::LUA_TFUNCTION => { ffi::lua_pushvalue(s, i); Ok(Value::Function(Function::from_stack(s)?)) },
                ffi::LUA_TUSERDATA => { ffi::lua_pushvalue(s, i); Ok(Value::UserData(AnyUserData::from_stack(s)?)) },
                _ => Err(Error::Type {
                    expected: "valor convertivel",
                    got: lua_type_name_at(s, i),
                }),
            }
        }
    }
}

pub struct SelfArgs<A>(pub AnyUserData, pub A);

pub trait FromLuaMulti: Sized {
    fn from_stack_multi(state: *mut lua_State, start: c_int) -> Result<Self>;
}

impl<A: FromLuaMulti> FromLuaMulti for SelfArgs<A> {
    fn from_stack_multi(state: *mut lua_State, start: c_int) -> Result<Self> {
        let ud = <AnyUserData as FromLua>::from_stack(state, start)?;
        let args = A::from_stack_multi(state, start + 1)?;
        Ok(SelfArgs(ud, args))
    }
}

impl FromLuaMulti for () {
    fn from_stack_multi(_: *mut lua_State, _: c_int) -> Result<Self> {
        Ok(())
    }
}

impl<A: FromLua> FromLuaMulti for A {
    fn from_stack_multi(s: *mut lua_State, st: c_int) -> Result<Self> {
        A::from_stack(s, st)
    }
}

impl<A: FromLua, B: FromLua> FromLuaMulti for (A, B) {
    fn from_stack_multi(s: *mut lua_State, st: c_int) -> Result<Self> {
        Ok((A::from_stack(s, st)?, B::from_stack(s, st + 1)?))
    }
}

impl<A: FromLua, B: FromLua, C: FromLua> FromLuaMulti for (A, B, C) {
    fn from_stack_multi(s: *mut lua_State, st: c_int) -> Result<Self> {
        Ok((
            A::from_stack(s, st)?,
            B::from_stack(s, st + 1)?,
            C::from_stack(s, st + 2)?,
        ))
    }
}

impl<A: FromLua, B: FromLua, C: FromLua, D: FromLua> FromLuaMulti for (A, B, C, D) {
    fn from_stack_multi(s: *mut lua_State, st: c_int) -> Result<Self> {
        Ok((
            A::from_stack(s, st)?,
            B::from_stack(s, st + 1)?,
            C::from_stack(s, st + 2)?,
            D::from_stack(s, st + 3)?,
        ))
    }
}

impl<A: FromLua, B: FromLua, C: FromLua, D: FromLua, E: FromLua> FromLuaMulti for (A, B, C, D, E) {
    fn from_stack_multi(s: *mut lua_State, st: c_int) -> Result<Self> {
        Ok((
            A::from_stack(s, st)?,
            B::from_stack(s, st + 1)?,
            C::from_stack(s, st + 2)?,
            D::from_stack(s, st + 3)?,
            E::from_stack(s, st + 4)?,
        ))
    }
}

impl<A: FromLua, B: FromLua, C: FromLua, D: FromLua, E: FromLua, F: FromLua> FromLuaMulti
    for (A, B, C, D, E, F)
{
    fn from_stack_multi(s: *mut lua_State, st: c_int) -> Result<Self> {
        Ok((
            A::from_stack(s, st)?,
            B::from_stack(s, st + 1)?,
            C::from_stack(s, st + 2)?,
            D::from_stack(s, st + 3)?,
            E::from_stack(s, st + 4)?,
            F::from_stack(s, st + 5)?,
        ))
    }
}

impl<A: FromLua, B: FromLua, C: FromLua, D: FromLua, E: FromLua, F: FromLua, G: FromLua>
    FromLuaMulti for (A, B, C, D, E, F, G)
{
    fn from_stack_multi(s: *mut lua_State, st: c_int) -> Result<Self> {
        Ok((
            A::from_stack(s, st)?,
            B::from_stack(s, st + 1)?,
            C::from_stack(s, st + 2)?,
            D::from_stack(s, st + 3)?,
            E::from_stack(s, st + 4)?,
            F::from_stack(s, st + 5)?,
            G::from_stack(s, st + 6)?,
        ))
    }
}

impl<
        A: FromLua,
        B: FromLua,
        C: FromLua,
        D: FromLua,
        E: FromLua,
        F: FromLua,
        G: FromLua,
        H: FromLua,
    > FromLuaMulti for (A, B, C, D, E, F, G, H)
{
    fn from_stack_multi(s: *mut lua_State, st: c_int) -> Result<Self> {
        Ok((
            A::from_stack(s, st)?,
            B::from_stack(s, st + 1)?,
            C::from_stack(s, st + 2)?,
            D::from_stack(s, st + 3)?,
            E::from_stack(s, st + 4)?,
            F::from_stack(s, st + 5)?,
            G::from_stack(s, st + 6)?,
            H::from_stack(s, st + 7)?,
        ))
    }
}

pub trait IntoLuaMulti {
    fn push_into_multi(self, state: *mut lua_State) -> Result<c_int>;
}

impl IntoLuaMulti for () {
    fn push_into_multi(self, _: *mut lua_State) -> Result<c_int> {
        Ok(0)
    }
}

impl<A: IntoLua> IntoLuaMulti for A {
    fn push_into_multi(self, s: *mut lua_State) -> Result<c_int> {
        self.push_into(s)?;
        Ok(1)
    }
}

impl<A: IntoLua, B: IntoLua> IntoLuaMulti for (A, B) {
    fn push_into_multi(self, s: *mut lua_State) -> Result<c_int> {
        self.0.push_into(s)?;
        self.1.push_into(s)?;
        Ok(2)
    }
}

impl<A: IntoLua, B: IntoLua, C: IntoLua> IntoLuaMulti for (A, B, C) {
    fn push_into_multi(self, s: *mut lua_State) -> Result<c_int> {
        self.0.push_into(s)?;
        self.1.push_into(s)?;
        self.2.push_into(s)?;
        Ok(3)
    }
}

impl<A: IntoLua, B: IntoLua, C: IntoLua, D: IntoLua> IntoLuaMulti for (A, B, C, D) {
    fn push_into_multi(self, s: *mut lua_State) -> Result<c_int> {
        self.0.push_into(s)?;
        self.1.push_into(s)?;
        self.2.push_into(s)?;
        self.3.push_into(s)?;
        Ok(4)
    }
}

impl<A: IntoLua, B: IntoLua, C: IntoLua, D: IntoLua, E: IntoLua> IntoLuaMulti for (A, B, C, D, E) {
    fn push_into_multi(self, s: *mut lua_State) -> Result<c_int> {
        self.0.push_into(s)?;
        self.1.push_into(s)?;
        self.2.push_into(s)?;
        self.3.push_into(s)?;
        self.4.push_into(s)?;
        Ok(5)
    }
}

impl<A: IntoLua, B: IntoLua, C: IntoLua, D: IntoLua, E: IntoLua, F: IntoLua> IntoLuaMulti
    for (A, B, C, D, E, F)
{
    fn push_into_multi(self, s: *mut lua_State) -> Result<c_int> {
        self.0.push_into(s)?;
        self.1.push_into(s)?;
        self.2.push_into(s)?;
        self.3.push_into(s)?;
        self.4.push_into(s)?;
        self.5.push_into(s)?;
        Ok(6)
    }
}

impl<A: IntoLua, B: IntoLua, C: IntoLua, D: IntoLua, E: IntoLua, F: IntoLua, G: IntoLua>
    IntoLuaMulti for (A, B, C, D, E, F, G)
{
    fn push_into_multi(self, s: *mut lua_State) -> Result<c_int> {
        self.0.push_into(s)?;
        self.1.push_into(s)?;
        self.2.push_into(s)?;
        self.3.push_into(s)?;
        self.4.push_into(s)?;
        self.5.push_into(s)?;
        self.6.push_into(s)?;
        Ok(7)
    }
}

impl<
        A: IntoLua,
        B: IntoLua,
        C: IntoLua,
        D: IntoLua,
        E: IntoLua,
        F: IntoLua,
        G: IntoLua,
        H: IntoLua,
    > IntoLuaMulti for (A, B, C, D, E, F, G, H)
{
    fn push_into_multi(self, s: *mut lua_State) -> Result<c_int> {
        self.0.push_into(s)?;
        self.1.push_into(s)?;
        self.2.push_into(s)?;
        self.3.push_into(s)?;
        self.4.push_into(s)?;
        self.5.push_into(s)?;
        self.6.push_into(s)?;
        self.7.push_into(s)?;
        Ok(8)
    }
}