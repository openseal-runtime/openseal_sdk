use openseal_sdk::prelude::*;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, VecDeque};

#[allow(dead_code)]
fn conversions(_lua: &Lua) -> Result<()> {
    let _s: Cow<'static, str> = Cow::Owned("hello".to_string());

    let _queue: VecDeque<i32> = VecDeque::from(vec![1, 2, 3]);

    let mut _map: HashMap<String, i32> = HashMap::new();
    _map.insert("a".to_string(), 1);

    let mut _bmap: BTreeMap<String, i32> = BTreeMap::new();
    _bmap.insert("b".to_string(), 2);

    Ok(())
}

fn main() {}