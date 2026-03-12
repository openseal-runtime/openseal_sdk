use crate::__ffi as ffi;
use crate::convert::FromLua;
use crate::metatable::{ensure_userdata_metatable, push_userdata_metatable};
use crate::registry::RegistryRef;
use crate::safety::{StackGuard, ensure_state, stack_dump};
use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_int;

pub use ffi::lua_State;

#[derive(Debug, Clone)]
pub enum Error {
    Runtime(String),
    Type { expected: &'static str, got: String },
    TypePath {
        expected: &'static str,
        got: String,
        path: String,
    },
    Borrow(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Runtime(msg) => write!(f, "{}", msg),
            Error::Type { expected, got } => {
                write!(f, "Erro de tipo: esperava {}, recebeu {}", expected, got)
            }
            Error::TypePath {
                expected,
                got,
                path,
            } => {
                write!(f, "Erro de tipo em {}: esperava {}, recebeu {}", path, expected, got)
            }
            Error::Borrow(msg) => write!(f, "Erro de borrow: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn with_path(self, path: impl Into<String>) -> Self {
        let path = path.into();
        match self {
            Error::Type { expected, got } => Error::TypePath {
                expected,
                got,
                path,
            },
            other => other,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy)]
pub struct Lua {
    pub(crate) state: *mut lua_State,
}

impl Lua {
    pub unsafe fn from_raw(state: *mut lua_State) -> Self {
        Self { state }
    }

    pub fn state(&self) -> *mut lua_State {
        self.state
    }

    pub fn stack_top(&self) -> Result<i32> {
        ensure_state(self.state)?;
        Ok(unsafe { ffi::lua_gettop(self.state) })
    }

    pub fn stack_dump(&self) -> Result<String> {
        stack_dump(self.state)
    }

    pub fn create_table(&self) -> Result<Table> {
        ensure_state(self.state)?;
        unsafe {
            ffi::lua_createtable(self.state, 0, 0);
            Table::from_stack(self.state)
        }
    }

    pub fn create_function<A, R, F>(&self, func: F) -> Result<Function>
    where
        A: crate::convert::FromLuaMulti + 'static,
        R: crate::convert::IntoLuaMulti + 'static,
        F: Fn(&Lua, A) -> Result<R> + 'static,
    {
        ensure_state(self.state)?;
        crate::closure::create_function(self.state, func)
    }

    pub fn create_userdata<T: 'static>(&self, data: T) -> Result<AnyUserData> {
        ensure_state(self.state)?;
        let _guard = StackGuard::new(self.state)?;

        unsafe {
            let _ = ffi::lua_newuserdata_t(
                self.state,
                UserDataCell {
                    value: RefCell::new(data),
                },
            );
            ensure_userdata_metatable::<T>(self.state)?;
            push_userdata_metatable::<T>(self.state)?;
            if ffi::lua_setmetatable(self.state, -2) == 0 {
                return Err(Error::Runtime("falha ao aplicar metatable do userdata".to_string()));
            }
            AnyUserData::from_stack(self.state)
        }
    }

    pub fn register_userdata_method<T: 'static>(&self, name: &str, func: Function) -> Result<()> {
        ensure_state(self.state)?;
        let _guard = StackGuard::new(self.state)?;

        ensure_userdata_metatable::<T>(self.state)?;
        push_userdata_metatable::<T>(self.state)?;

        let key = CString::new(name)
            .map_err(|_| Error::Runtime("nome de metodo invalido".to_string()))?;

        unsafe {
            func.push_to_stack();
            ffi::lua_setfield(self.state, -2, key.as_ptr());
        }

        Ok(())
    }

    pub fn userdata_methods<T: 'static>(&self) -> crate::userdata_methods::UserDataMethods<'_, T> {
        crate::userdata_methods::UserDataMethods::new(self)
    }
}

pub struct Table {
    pub(crate) inner: RegistryRef,
}

impl Table {
    pub(crate) fn from_stack(state: *mut lua_State) -> Result<Self> {
        ensure_state(state)?;
        Ok(Self {
            inner: RegistryRef::new_from_top(state),
        })
    }

    pub fn state(&self) -> *mut lua_State {
        self.inner.state()
    }

    pub fn set(
        &self,
        key: impl crate::convert::IntoLua,
        value: impl crate::convert::IntoLua,
    ) -> Result<()> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        unsafe {
            self.push_to_stack();
            key.push_into(self.state())?;
            value.push_into(self.state())?;
            ffi::lua_settable(self.state(), -3);
        }
        Ok(())
    }

    pub fn get<T: crate::convert::FromLua>(&self, key: impl crate::convert::IntoLua) -> Result<T> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        unsafe {
            self.push_to_stack();
            key.push_into(self.state())?;
            ffi::lua_gettable(self.state(), -2);
            T::from_stack(self.state(), -1)
        }
    }

    pub fn raw_set(
        &self,
        key: impl crate::convert::IntoLua,
        value: impl crate::convert::IntoLua,
    ) -> Result<()> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        unsafe {
            self.push_to_stack();
            key.push_into(self.state())?;
            value.push_into(self.state())?;
            ffi::lua_rawset(self.state(), -3);
        }
        Ok(())
    }

    pub fn raw_get<T: crate::convert::FromLua>(&self, key: impl crate::convert::IntoLua) -> Result<T> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        unsafe {
            self.push_to_stack();
            key.push_into(self.state())?;
            ffi::lua_rawget(self.state(), -2);
            T::from_stack(self.state(), -1)
        }
    }

    pub fn set_field(&self, key: &str, value: impl crate::convert::IntoLua) -> Result<()> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        let c_key = CString::new(key).map_err(|_| Error::Runtime("chave invalida".to_string()))?;

        unsafe {
            self.push_to_stack();
            value.push_into(self.state())?;
            ffi::lua_setfield(self.state(), -2, c_key.as_ptr());
        }

        Ok(())
    }

    pub fn get_field<T: crate::convert::FromLua>(&self, key: &str) -> Result<T> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        let c_key = CString::new(key).map_err(|_| Error::Runtime("chave invalida".to_string()))?;

        unsafe {
            self.push_to_stack();
            ffi::lua_getfield(self.state(), -1, c_key.as_ptr());
            T::from_stack(self.state(), -1).map_err(|e| e.with_path(format!("table.{}", key)))
        }
    }

    pub fn contains_key(&self, key: impl crate::convert::IntoLua) -> Result<bool> {
        let value: Value = self.get(key)?;
        Ok(!matches!(value, Value::Nil))
    }

    pub fn len(&self) -> Result<usize> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        unsafe {
            self.push_to_stack();
            Ok(ffi::lua_objlen(self.state(), -1))
        }
    }

    pub fn clear(&self) -> Result<()> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        unsafe {
            self.push_to_stack();
            ffi::lua_cleartable(self.state(), -1);
        }
        Ok(())
    }

    pub fn iter(&self) -> Result<TableIter> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;

        let mut entries = Vec::new();
        unsafe {
            self.push_to_stack();
            let table_idx = ffi::lua_absindex(self.state(), -1);
            ffi::lua_pushnil(self.state());

            while ffi::lua_next(self.state(), table_idx) != 0 {
                ffi::lua_pushvalue(self.state(), -2);
                let key = Value::from_stack(self.state(), -1)?;
                ffi::lua_pop(self.state(), 1);

                let value = Value::from_stack(self.state(), -1)?;
                entries.push((key, value));

                ffi::lua_pop(self.state(), 1);
            }
        }

        Ok(TableIter {
            inner: entries.into_iter(),
        })
    }

    pub fn set_metamethod(&self, name: &str, func: Function) -> Result<()> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;
        let c_name = CString::new(name)
            .map_err(|_| Error::Runtime("nome de metamethod invalido".to_string()))?;

        unsafe {
            self.push_to_stack();
            func.push_to_stack();
            ffi::lua_setfield(self.state(), -2, c_name.as_ptr());
        }

        Ok(())
    }

    pub fn set_metatable(&self, metatable: &Table) -> Result<()> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;

        unsafe {
            self.push_to_stack();
            metatable.push_to_stack();
            if ffi::lua_setmetatable(self.state(), -2) == 0 {
                return Err(Error::Runtime("falha ao definir metatable".to_string()));
            }
        }

        Ok(())
    }

    pub fn get_metatable(&self) -> Result<Option<Table>> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;

        unsafe {
            self.push_to_stack();
            if ffi::lua_getmetatable(self.state(), -1) == 0 {
                return Ok(None);
            }
            Table::from_stack(self.state()).map(Some)
        }
    }

    pub fn set_readonly(&self, readonly: bool) -> Result<()> {
        ensure_state(self.state())?;
        let _guard = StackGuard::new(self.state())?;

        unsafe {
            self.push_to_stack();
            ffi::lua_setreadonly(self.state(), -1, if readonly { 1 } else { 0 });
        }

        Ok(())
    }

    pub fn push_to_stack(&self) {
        self.inner.push();
    }
}

pub struct TableIter {
    inner: std::vec::IntoIter<(Value, Value)>,
}

impl Iterator for TableIter {
    type Item = (Value, Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct Function {
    pub(crate) inner: RegistryRef,
}

impl Function {
    pub(crate) fn from_stack(state: *mut lua_State) -> Result<Self> {
        ensure_state(state)?;
        Ok(Self {
            inner: RegistryRef::new_from_top(state),
        })
    }

    pub fn push_to_stack(&self) {
        self.inner.push();
    }
}

pub struct AnyUserData {
    pub(crate) inner: RegistryRef,
}

impl AnyUserData {
    pub(crate) fn from_stack(state: *mut lua_State) -> Result<Self> {
        ensure_state(state)?;
        Ok(Self {
            inner: RegistryRef::new_from_top(state),
        })
    }

    pub fn push_to_stack(&self) {
        self.inner.push();
    }

    pub fn with_ref<T: 'static, R>(&self, f: impl FnOnce(&T) -> R) -> Result<R> {
        let state = self.inner.state();
        ensure_state(state)?;
        let _guard = StackGuard::new(state)?;

        unsafe {
            self.push_to_stack();
            assert_userdata_type::<T>(state, -1)?;
            let ptr = ffi::lua_touserdata(state, -1) as *const UserDataCell<T>;
            if ptr.is_null() {
                return Err(Error::Runtime("userdata invalido".to_string()));
            }
            let borrowed = (*ptr)
                .value
                .try_borrow()
                .map_err(|_| Error::Borrow("userdata ja mutavelmente emprestado".to_string()))?;
            Ok(f(&*borrowed))
        }
    }

    pub fn with_mut<T: 'static, R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R> {
        let state = self.inner.state();
        ensure_state(state)?;
        let _guard = StackGuard::new(state)?;

        unsafe {
            self.push_to_stack();
            assert_userdata_type::<T>(state, -1)?;
            let ptr = ffi::lua_touserdata(state, -1) as *mut UserDataCell<T>;
            if ptr.is_null() {
                return Err(Error::Runtime("userdata invalido".to_string()));
            }
            let mut borrowed = (*ptr)
                .value
                .try_borrow_mut()
                .map_err(|_| Error::Borrow("userdata ja emprestado".to_string()))?;
            Ok(f(&mut *borrowed))
        }
    }

    pub fn set_metatable<T: 'static>(&self) -> Result<()> {
        let state = self.inner.state();
        ensure_state(state)?;
        let _guard = StackGuard::new(state)?;

        ensure_userdata_metatable::<T>(state)?;
        unsafe {
            self.push_to_stack();
            push_userdata_metatable::<T>(state)?;
            if ffi::lua_setmetatable(state, -2) == 0 {
                return Err(Error::Runtime("falha ao definir metatable do userdata".to_string()));
            }
        }

        Ok(())
    }
}

#[repr(C)]
struct UserDataCell<T> {
    value: RefCell<T>,
}

unsafe fn assert_userdata_type<T: 'static>(state: *mut lua_State, idx: c_int) -> Result<()> {
    if ffi::lua_type(state, idx) != ffi::LUA_TUSERDATA {
        return Err(Error::Type {
            expected: "userdata",
            got: lua_type_name_at(state, idx),
        });
    }

    if ffi::lua_getmetatable(state, idx) == 0 {
        return Err(Error::Runtime("userdata sem metatable".to_string()));
    }

    push_userdata_metatable::<T>(state)?;
    let same = ffi::lua_rawequal(state, -1, -2) != 0;
    ffi::lua_pop(state, 2);

    if !same {
        return Err(Error::Type {
            expected: "userdata do tipo registrado",
            got: "userdata de outro tipo".to_string(),
        });
    }

    Ok(())
}

#[allow(dead_code)]
pub(crate) unsafe fn lua_type_name_at(state: *mut lua_State, idx: c_int) -> String {
    let tid = ffi::lua_type(state, idx);
    let name_ptr = ffi::lua_typename(state, tid);
    if name_ptr.is_null() {
        return "unknown".to_string();
    }
    std::ffi::CStr::from_ptr(name_ptr)
        .to_string_lossy()
        .into_owned()
}

pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Table(Table),
    Function(Function),
    UserData(AnyUserData),
}

impl Table {
    pub fn into_owned(self) -> OwnedTable {
        OwnedTable(self)
    }
}

impl Function {
    pub fn into_owned(self) -> OwnedFunction {
        OwnedFunction(self)
    }
}

pub struct OwnedTable(pub(crate) Table);

impl OwnedTable {
    pub fn as_table(&self) -> &Table {
        &self.0
    }

    pub fn into_inner(self) -> Table {
        self.0
    }

    pub fn push_to_stack(&self) {
        self.0.push_to_stack();
    }
}

pub struct OwnedFunction(pub(crate) Function);

impl OwnedFunction {
    pub fn as_function(&self) -> &Function {
        &self.0
    }

    pub fn into_inner(self) -> Function {
        self.0
    }

    pub fn push_to_stack(&self) {
        self.0.push_to_stack();
    }
}