use openseal_sdk::prelude::*;

#[derive(Default)]
struct Counter {
    value: i32,
}

#[allow(dead_code)]
fn register_userdata(lua: &Lua) -> Result<()> {
    let methods = lua.userdata_methods::<Counter>();

    methods.add_method("get", |_lua, this, (): ()| Ok(this.value))?;
    methods.add_method_mut("inc", |_lua, this, amount: i32| {
        this.value += amount;
        Ok(this.value)
    })?;

    methods.add_meta_method(MetaMethod::ToString, |_lua, this, (): ()| {
        Ok(format!("Counter({})", this.value))
    })?;

    Ok(())
}

fn main() {}