// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use std::io::Write;
use toml_edit::{Item, Key, Table, Value};

/// Resolves the given key in the given root table and prints its sub-keys.
/// If en error happens during resolving or the key doesn't resolve to a table or array, an error is printed instead.
pub fn list_value(key: Option<String>, root: &Table, sep: char) {
    match key {
        Some(key) => {
            let Ok(keys) = Key::parse(&key) else {
                println!("ERROR: Key '{key}' is invalid");
                return;
            };

            match keys.get(0) {
                Some(segment) => list_table_keys(root, segment, &keys[1..], sep),
                None => println!("ERROR: Key is empty"),
            }
        }
        None => print_table_keys(root, sep),
    }
}

fn list_value_keys(value: &Value, keys: &[Key], sep: char) {
    match value {
        Value::Array(array) => {
            let segment = match keys.get(0) {
                Some(segment) => segment,
                None => {
                    print_array_keys(array.len(), sep);
                    return;
                }
            };

            let idx = match segment.parse::<usize>() {
                Ok(idx) => idx,
                Err(err) => {
                    println!("ERROR: Cannot parse array index for segment '{segment}': {err}");
                    return;
                }
            };

            match array.get(idx) {
                Some(value) => list_value_keys(value, &keys[1..], sep),
                None => println!("ERROR: Key segment '{segment}' is missing"),
            }
        }
        Value::InlineTable(table) => {
            let segment = match keys.get(0) {
                Some(segment) => segment,
                None => {
                    let mut lock = stdout();

                    for (idx, (key, _)) in table.iter().enumerate() {
                        if idx != 0 {
                            let _ = write!(lock, "{}", sep);
                        }

                        let _ = write!(lock, "{}", key);
                    }

                    let _ = writeln!(lock);
                    return;
                }
            };

            match table.get(segment) {
                Some(value) => list_value_keys(value, &keys[1..], sep),
                None => println!("ERROR: Key segment '{segment}' is missing"),
            }
        }
        _ => println!("ERROR: Key segment '{}' is a value", &keys[0]),
    }
}

fn list_table_keys(table: &Table, segment: &str, keys: &[Key], sep: char) {
    match table.get(segment) {
        Some(Item::Value(value)) => list_value_keys(value, keys, sep),
        Some(Item::Table(table)) => match keys.get(0) {
            Some(segment) => list_table_keys(table, segment, &keys[1..], sep),
            None => print_table_keys(table, sep),
        },
        Some(Item::ArrayOfTables(array)) => {
            let segment = match keys.get(0) {
                Some(segment) => segment,
                None => {
                    print_array_keys(array.len(), sep);
                    return;
                }
            };

            let idx = match segment.parse::<usize>() {
                Ok(idx) => idx,
                Err(err) => {
                    println!("ERROR: Cannot parse array index for segment '{segment}': {err}");
                    return;
                }
            };

            let table = match array.get(idx) {
                Some(table) => table,
                None => {
                    println!("ERROR: Key segment '{segment}' is missing");
                    return;
                }
            };

            match keys.get(1) {
                Some(segment) => list_table_keys(table, segment, &keys[2..], sep),
                None => print_table_keys(table, sep),
            }
        }
        None | Some(Item::None) => println!("ERROR: Key segment '{segment}' is missing"),
    }
}

fn print_array_keys(len: usize, sep: char) {
    let mut lock = stdout();

    for i in 0..len {
        if i != 0 {
            let _ = write!(lock, "{}", sep);
        }

        let _ = write!(lock, "{}", i);
    }

    let _ = writeln!(lock);
}

fn print_table_keys(table: &Table, sep: char) {
    let mut lock = stdout();

    for (idx, (key, _)) in table.iter().enumerate() {
        if idx != 0 {
            let _ = write!(lock, "{}", sep);
        }

        let _ = write!(lock, "{}", key);
    }

    let _ = writeln!(lock);
}

#[cfg(not(test))]
fn stdout() -> std::io::StdoutLock<'static> {
    std::io::stdout().lock()
}

#[cfg(test)]
fn stdout() -> std::sync::MutexGuard<'static, Vec<u8>> {
    crate::testing::TESTING_STDOUT.lock().unwrap()
}

#[cfg(test)]
mod test {
    use crate::testing::{assert_error, assert_output};
    use toml_edit::Document;

    #[test]
    fn test_list_value() {
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

        super::list_value(None, document.as_table_mut(), ' ');
        assert_output("dragon kobold");

        super::list_value(Some("dragon".into()), document.as_table_mut(), ' ');
        assert_output("rawr size loves");

        super::list_value(Some("kobold".into()), document.as_table_mut(), ',');
        assert_output("has dragon,says");

        super::list_value(Some("kobold.says".into()), document.as_table_mut(), ' ');
        assert_output("0 1 2");

        super::list_value(Some("kobold.says.0".into()), document.as_table_mut(), ' ');
        assert_output("yip");

        super::list_value(Some("dragon.missing".into()), document.as_table_mut(), ' ');
        assert_error();

        super::list_value(
            Some("dragon.\"invalid key".into()),
            document.as_table_mut(),
            ' ',
        );
        assert_error();

        super::list_value(
            Some("dragon.'invalid key".into()),
            document.as_table_mut(),
            ' ',
        );
        assert_error();

        super::list_value(
            Some("dragon.invalid key".into()),
            document.as_table_mut(),
            ' ',
        );
        assert_error();
    }
}
