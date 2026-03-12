use crate::__ffi as ffi;
use crate::safety::push_lua_error;
use crate::types::*;
use std::os::raw::c_int;

#[repr(C)]
struct CallbackCell<F> {
    func: F,
}

pub fn create_function<A, R, F>(state: *mut lua_State, func: F) -> Result<Function>
where
    A: crate::convert::FromLuaMulti + 'static,
    R: crate::convert::IntoLuaMulti + 'static,
    F: Fn(&Lua, A) -> Result<R> + 'static,
{
    unsafe {
        let _cell_ptr = ffi::lua_newuserdata_t(state, CallbackCell { func });
        ffi::lua_pushcclosure(state, closure_wrapper::<A, R, F>, 1);
        Function::from_stack(state)
    }
}

unsafe extern "C-unwind" fn closure_wrapper<A, R, F>(state: *mut lua_State) -> c_int
where
    A: crate::convert::FromLuaMulti + 'static,
    R: crate::convert::IntoLuaMulti + 'static,
    F: Fn(&Lua, A) -> Result<R> + 'static,
{
    let upvalue_ptr = ffi::lua_touserdata(state, ffi::lua_upvalueindex(1));
    if upvalue_ptr.is_null() {
        push_lua_error(state, "falha interna: closure sem upvalue");
    }

    let cell = &*(upvalue_ptr as *const CallbackCell<F>);
    let lua = Lua::from_raw(state);

    let args = match A::from_stack_multi(state, 1) {
        Ok(args) => args,
        Err(e) => push_lua_error(state, &e.to_string()),
    };

    let call_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (cell.func)(&lua, args)
    }));

    match call_result {
        Ok(Ok(ret)) => match ret.push_into_multi(state) {
            Ok(count) => count,
            Err(e) => push_lua_error(state, &e.to_string()),
        },
        Ok(Err(e)) => push_lua_error(state, &e.to_string()),
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                format!("panic em callback nativa: {s}")
            } else if let Some(s) = payload.downcast_ref::<String>() {
                format!("panic em callback nativa: {s}")
            } else {
                "panic em callback nativa sem mensagem".to_string()
            };
            push_lua_error(state, &msg)
        }
    }
}