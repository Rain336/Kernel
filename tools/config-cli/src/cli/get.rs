// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::utils;
#[cfg(test)]
use std::io::Write;
use toml_edit::{Key, Table, Value};

/// Resolves the given key in the given root table and prints it as `<key> = <value>`.
/// If en error happens during resolving, it is printed instead.
pub fn read_value(key: String, root: &mut Table) {
    let Ok(keys) = Key::parse(&key) else {
        println!("ERROR: Key '{key}' is invalid");
        return;
    };

    let Some(segment) = keys.get(0) else {
        println!("ERROR: Key is empty");
        return;
    };

    let Some(value) = utils::get_value_from_table(root, segment, &keys[1..], &key) else {
        return;
    };

    match value {
        Value::String(str) => println!("{key} = {}", str.display_repr()),
        Value::Integer(int) => println!("{key} = {}", int.display_repr()),
        Value::Float(float) => println!("{key} = {}", float.display_repr()),
        Value::Boolean(boolean) => println!("{key} = {}", boolean.display_repr()),
        Value::Datetime(dt) => println!("{key} = {}", dt.display_repr()),
        Value::Array(array) => println!("{key} = {array}"),
        Value::InlineTable(table) => println!("{key} = {table}"),
    }
}

#[cfg(test)]
mod test {
    use crate::testing::{assert_error, assert_output};
    use toml_edit::Document;

    #[test]
    fn test_read_value() {
        let toml = r#"
[dragon]
rawr = true
size = 420
loves = [ "hugs", "kisses", "cuddles" ]

[kobold]
"has dragon" = true
says = [ { yip = 5 }, { yap = "too many" }, { yop = false } ]
"#;
        let mut document = toml.parse::<Document>().unwrap();

        super::read_value("dragon.rawr".into(), document.as_table_mut());
        assert_output("dragon.rawr = true");

        super::read_value("dragon.size".into(), document.as_table_mut());
        assert_output("dragon.size = 420");

        super::read_value("dragon.loves.0".into(), document.as_table_mut());
        assert_output("dragon.loves.0 = \"hugs\"");

        super::read_value("dragon.loves.2".into(), document.as_table_mut());
        assert_output("dragon.loves.2 = \"cuddles\"");

        super::read_value("kobold.\"has dragon\"".into(), document.as_table_mut());
        assert_output("kobold.\"has dragon\" = true");

        super::read_value("kobold.'has dragon'".into(), document.as_table_mut());
        assert_output("kobold.'has dragon' = true");

        super::read_value("kobold.says.0.yip".into(), document.as_table_mut());
        assert_output("kobold.says.0.yip = 5");

        super::read_value("kobold.says.1.yap".into(), document.as_table_mut());
        assert_output("kobold.says.1.yap = \"too many\"");

        super::read_value("kobold.says.2.yop".into(), document.as_table_mut());
        assert_output("kobold.says.2.yop = false");

        super::read_value("dragon.stinky".into(), document.as_table_mut());
        assert_error();

        super::read_value("dragon.'broken string".into(), document.as_table_mut());
        assert_error();

        super::read_value("dragon.\"broken string".into(), document.as_table_mut());
        assert_error();

        super::read_value("dragon.broken string".into(), document.as_table_mut());
        assert_error();
    }
}
