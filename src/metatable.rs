use crate::__ffi as ffi;
use crate::safety::ensure_state;
use crate::types::{Error, Result, lua_State};

pub(crate) fn ud_registry_key() -> *const std::ffi::c_void {
    ud_registry_key as *const std::ffi::c_void
}

pub(crate) fn type_registry_key<T: 'static>() -> *const std::ffi::c_void {
    type_registry_key::<T> as *const std::ffi::c_void
}

pub(crate) fn ensure_userdata_metatable<T: 'static>(state: *mut lua_State) -> Result<()> {
    ensure_state(state)?;

    unsafe {
        ffi::lua_rawgetptagged(state, ffi::LUA_REGISTRYINDEX, ud_registry_key(), 0);
        if ffi::lua_type(state, -1) == ffi::LUA_TNIL {
            ffi::lua_pop(state, 1);
            ffi::lua_newtable(state);
            ffi::lua_rawsetptagged(state, ffi::LUA_REGISTRYINDEX, ud_registry_key(), 0);
            ffi::lua_rawgetptagged(state, ffi::LUA_REGISTRYINDEX, ud_registry_key(), 0);
        }

        ffi::lua_rawgetptagged(state, -1, type_registry_key::<T>(), 0);
        if ffi::lua_type(state, -1) != ffi::LUA_TNIL {
            ffi::lua_pop(state, 2);
            return Ok(());
        }

        ffi::lua_pop(state, 1);
        ffi::lua_newtable(state);

        let mt_key = std::ffi::CString::new("__index")
            .map_err(|_| Error::Runtime("chave invalida".to_string()))?;
        ffi::lua_pushvalue(state, -1);
        ffi::lua_setfield(state, -2, mt_key.as_ptr());

        ffi::lua_pushvalue(state, -1);
        ffi::lua_rawsetptagged(state, -3, type_registry_key::<T>(), 0);

        ffi::lua_pop(state, 2);
    }

    Ok(())
}

pub(crate) fn push_userdata_metatable<T: 'static>(state: *mut lua_State) -> Result<()> {
    ensure_state(state)?;

    unsafe {
        ffi::lua_rawgetptagged(state, ffi::LUA_REGISTRYINDEX, ud_registry_key(), 0);
        if ffi::lua_type(state, -1) == ffi::LUA_TNIL {
            ffi::lua_pop(state, 1);
            return Err(Error::Runtime("registro de userdata ausente".to_string()));
        }

        ffi::lua_rawgetptagged(state, -1, type_registry_key::<T>(), 0);
        ffi::lua_remove(state, -2);

        if ffi::lua_type(state, -1) == ffi::LUA_TNIL {
            ffi::lua_pop(state, 1);
            return Err(Error::Runtime("metatable de userdata nao registrada".to_string()));
        }
    }

    Ok(())
}