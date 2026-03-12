use crate::__ffi as ffi;
use crate::types::lua_State;
use std::os::raw::c_int;

pub struct RegistryRef {
    state: *mut lua_State,
    ref_id: c_int,
}

impl RegistryRef {
    pub fn new_from_top(state: *mut lua_State) -> Self {
        unsafe {
            let ref_id = ffi::lua_ref(state, -1);
            ffi::lua_settop(state, ffi::lua_gettop(state) - 1);
            Self { state, ref_id }
        }
    }

    pub fn push(&self) {
        unsafe {
            ffi::lua_rawgeti_(self.state, ffi::LUA_REGISTRYINDEX, self.ref_id);
        }
    }

    pub fn state(&self) -> *mut lua_State {
        self.state
    }
}

impl Drop for RegistryRef {
    fn drop(&mut self) {
        if !self.state.is_null() {
            unsafe {
                ffi::lua_unref(self.state, self.ref_id);
            }
        }
    }
}