use crate::__ffi as ffi;
use crate::types::{Error, Result, lua_State};

pub fn ensure_state(state: *mut lua_State) -> Result<()> {
    if state.is_null() {
        return Err(Error::Runtime("lua_State null".to_string()));
    }
    Ok(())
}

pub struct StackGuard {
    state: *mut lua_State,
    top: i32,
}

impl StackGuard {
    pub fn new(state: *mut lua_State) -> Result<Self> {
        ensure_state(state)?;
        let top = unsafe { ffi::lua_gettop(state) };
        Ok(Self { state, top })
    }
}

impl Drop for StackGuard {
    fn drop(&mut self) {
        if !self.state.is_null() {
            unsafe {
                ffi::lua_settop(self.state, self.top);
            }
        }
    }
}

pub fn stack_dump(state: *mut lua_State) -> Result<String> {
    ensure_state(state)?;
    let top = unsafe { ffi::lua_gettop(state) };
    let mut out = String::new();
    out.push_str(&format!("stack_top={top}"));

    for i in 1..=top {
        let typ = unsafe {
            let tid = ffi::lua_type(state, i);
            let name_ptr = ffi::lua_typename(state, tid);
            if name_ptr.is_null() {
                "unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(name_ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        };
        out.push_str(&format!("\n[{i}] {typ}"));
    }

    Ok(out)
}

pub unsafe fn push_lua_error(state: *mut lua_State, msg: &str) -> ! {
    let bytes = msg.as_bytes();
    ffi::lua_pushlstring_(state, bytes.as_ptr() as *const _, bytes.len());
    ffi::lua_error(state);
}