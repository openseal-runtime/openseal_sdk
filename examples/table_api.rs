use openseal_sdk::prelude::*;

#[allow(dead_code)]
fn table_api(lua: &Lua) -> Result<()> {
    let tbl = lua.create_table()?;
    tbl.set("name", "openseal")?;
    tbl.set_field("version", 1_i32)?;

    let _name: String = tbl.get("name")?;
    let _version: i32 = tbl.get_field("version")?;

    let _has_name = tbl.contains_key("name")?;
    let _len = tbl.len()?;

    for (_k, _v) in tbl.iter()? {
        // iterate all pairs
    }

    tbl.clear()?;
    Ok(())
}

fn main() {}