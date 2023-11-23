// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::utils;
#[cfg(test)]
use std::io::Write;
use toml_edit::{Key, Table, Value};

/// Resolves the given key in the given root table, sets it to the given new_value and prints it as `<key> = <new_value>`.
/// If en error happens during resolving or parsing the new_value, it is printed instead.
pub fn write_value(key: String, new_value: String, root: &mut Table) {
    let keys = match Key::parse(&key) {
        Ok(keys) => keys,
        Err(err) => {
            println!("ERROR: Key '{key}' is invalid: {err}");
            return;
        }
    };

    let Some(segment) = keys.get(0) else {
        println!("ERROR: Key is empty");
        return;
    };

    let Some(value) = utils::get_value_from_table(root, segment, &keys[1..], &key) else {
        return;
    };

    let new_value = match new_value.parse::<Value>() {
        Ok(new_value) => new_value,
        Err(err) => {
            println!("ERROR: Cannot parse new value '{new_value}' for key '{key}': {err}");
            return;
        }
    };

    match (value, new_value) {
        (Value::String(str), Value::String(new_str)) => {
            *str = new_str;
            println!("{key} = {}", str.display_repr());
        }
        (Value::Integer(int), Value::Integer(new_int)) => {
            *int = new_int;
            println!("{key} = {}", int.display_repr());
        }
        (Value::Float(float), Value::Float(new_float)) => {
            *float = new_float;
            println!("{key} = {}", float.display_repr());
        }
        (Value::Boolean(boolean), Value::Boolean(new_boolean)) => {
            *boolean = new_boolean;
            println!("{key} = {}", boolean.display_repr());
        }
        (Value::Datetime(dt), Value::Datetime(new_dt)) => {
            *dt = new_dt;
            println!("{key} = {}", dt.display_repr());
        }
        (Value::Array(array), Value::Array(new_array)) => {
            *array = new_array;
            println!("{key} = {array}");
        }
        (Value::InlineTable(array), Value::InlineTable(new_array)) => {
            *array = new_array;
            println!("{key} = {array}");
        }
        (left, right) => {
            println!(
                "ERROR: The type of the value in Cargo.toml ({}) and the supplied type ({}) do not match.",
                left.type_name(),
                right.type_name()
            );
        }
    }
}

#[cfg(test)]
mod test {
    use crate::testing::{assert_error, assert_output};
    use toml_edit::Document;

    #[test]
    fn test_write_value() {
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

        super::write_value(
            "dragon.rawr".into(),
            "false".into(),
            document.as_table_mut(),
        );
        assert_output("dragon.rawr = false");

        super::write_value("dragon.size".into(), "600".into(), document.as_table_mut());
        assert_output("dragon.size = 600");

        super::write_value(
            "dragon.loves.0".into(),
            "'huggies'".into(),
            document.as_table_mut(),
        );
        assert_output("dragon.loves.0 = 'huggies'");

        super::write_value(
            "dragon.loves.2".into(),
            "''".into(),
            document.as_table_mut(),
        );
        assert_output("dragon.loves.2 = ''");

        super::write_value(
            "kobold.\"has dragon\"".into(),
            "false".into(),
            document.as_table_mut(),
        );
        assert_output("kobold.\"has dragon\" = false");

        super::write_value(
            "kobold.'has dragon'".into(),
            "true".into(),
            document.as_table_mut(),
        );
        assert_output("kobold.'has dragon' = true");

        super::write_value(
            "kobold.says.0.yip".into(),
            "-500".into(),
            document.as_table_mut(),
        );
        assert_output("kobold.says.0.yip = -500");

        super::write_value(
            "kobold.says.1.yap".into(),
            "'got less'".into(),
            document.as_table_mut(),
        );
        assert_output("kobold.says.1.yap = 'got less'");

        super::write_value(
            "kobold.says.2.yop".into(),
            "true".into(),
            document.as_table_mut(),
        );
        assert_output("kobold.says.2.yop = true");

        super::write_value(
            "dragon.stinky".into(),
            String::new(),
            document.as_table_mut(),
        );
        assert_error();

        super::write_value(
            "dragon.'broken string".into(),
            String::new(),
            document.as_table_mut(),
        );
        assert_error();

        super::write_value(
            "dragon.\"broken string".into(),
            String::new(),
            document.as_table_mut(),
        );
        assert_error();

        super::write_value(
            "dragon.broken string".into(),
            String::new(),
            document.as_table_mut(),
        );
        assert_error();

        super::write_value(
            "kobold.says.2.yop".into(),
            "5".into(),
            document.as_table_mut(),
        );
        assert_error();

        super::write_value(
            "dragon.rawr".into(),
            "very much".into(),
            document.as_table_mut(),
        );
        assert_error();
    }
}
