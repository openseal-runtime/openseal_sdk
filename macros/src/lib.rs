use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Marca a funcao como o ponto de entrada nativo do OpenSeal.
/// Gera a funcao `luaopen_<nome>` que o Runtime carrega via `loadlib`.
#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let luaopen_name = syn::Ident::new(&format!("luaopen_{}", fn_name), fn_name.span());

    let expanded = quote! {
        #[unsafe(no_mangle)]
        pub static OPENSEAL_ABI_VERSION: u32 = openseal_sdk::ABI_VERSION;

        #[unsafe(no_mangle)]
        pub unsafe extern "C-unwind" fn #luaopen_name(
            __state: *mut openseal_sdk::lua_State,
        ) -> std::ffi::c_int {
            let __lua = unsafe { openseal_sdk::Lua::from_raw(__state) };

            let __result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                #fn_name(&__lua)
            }));

            match __result {
                Ok(Ok(table)) => {
                    let table = std::mem::ManuallyDrop::new(table);
                    table.push_to_stack();
                    1
                }
                Ok(Err(err)) => {
                    let msg = err.to_string();
                    unsafe {
                        openseal_sdk::__ffi::lua_pushlstring_(
                            __state,
                            msg.as_ptr() as *const std::os::raw::c_char,
                            msg.len(),
                        );
                        openseal_sdk::__ffi::lua_error(__state);
                    }
                }
                Err(payload) => {
                    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                        format!("panic no modulo nativo: {s}")
                    } else if let Some(s) = payload.downcast_ref::<String>() {
                        format!("panic no modulo nativo: {s}")
                    } else {
                        "panic no modulo nativo sem mensagem".to_string()
                    };

                    unsafe {
                        openseal_sdk::__ffi::lua_pushlstring_(
                            __state,
                            msg.as_ptr() as *const std::os::raw::c_char,
                            msg.len(),
                        );
                        openseal_sdk::__ffi::lua_error(__state);
                    }
                }
            }
        }

        #input
    };

    expanded.into()
}