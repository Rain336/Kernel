// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#[cfg(test)]
use std::io::Write;
use toml_edit::{Item, Key, Table, Value};

fn get_value_from_value<'a>(
    value: &'a mut Value,
    segment: &str,
    keys: &[Key],
) -> Option<&'a mut Value> {
    if keys.is_empty() {
        return Some(value);
    }

    match value {
        Value::Array(array) => {
            let segment = keys.get(0).unwrap();

            let idx = match segment.parse::<usize>() {
                Ok(idx) => idx,
                Err(err) => {
                    println!("ERROR: Cannot parse array index for segment '{segment}': {err}");
                    return None;
                }
            };

            match array.get_mut(idx) {
                Some(value) => get_value_from_value(value, segment, &keys[1..]),
                None => {
                    println!("ERROR: Key segment '{segment}' is missing");
                    None
                }
            }
        }
        Value::InlineTable(table) => {
            let segment = keys.get(0).unwrap();

            match table.get_mut(segment) {
                Some(value) => get_value_from_value(value, segment, &keys[1..]),
                None => {
                    println!("ERROR: Key segment '{segment}' is missing");
                    None
                }
            }
        }
        _ => {
            println!("Key at segment {segment} is an {}", value.type_name());
            None
        }
    }
}

fn get_value_from_item<'a>(
    item: &'a mut Item,
    segment: &str,
    keys: &[Key],
    key: &str,
) -> Option<&'a mut Value> {
    match item {
        Item::Value(value) => {
            if keys.is_empty() {
                Some(value)
            } else {
                get_value_from_value(value, segment, keys)
            }
        }
        Item::Table(table) => match keys.get(0) {
            Some(segment) => get_value_from_table(table, segment, &keys[1..], key),
            None => {
                println!("ERROR: Key ends at a table");
                None
            }
        },
        Item::ArrayOfTables(array) => {
            let segment = match keys.get(0) {
                Some(segment) => segment,
                None => {
                    println!("ERROR: Key ends at an array of tables");
                    return None;
                }
            };

            let idx = match segment.parse::<usize>() {
                Ok(idx) => idx,
                Err(err) => {
                    println!("ERROR: Cannot parse array index for segment '{segment}': {err}");
                    return None;
                }
            };

            let table = match array.get_mut(idx) {
                Some(table) => table,
                None => {
                    println!("ERROR: Key segment '{segment}' is missing");
                    return None;
                }
            };

            match keys.get(1) {
                Some(segment) => get_value_from_table(table, segment, &keys[2..], key),
                None => {
                    println!("ERROR: Key ends at a table");
                    None
                }
            }
        }
        Item::None => {
            println!("ERROR: Key segment '{segment}' is missing");
            None
        }
    }
}

pub fn get_value_from_table<'a>(
    table: &'a mut Table,
    segment: &str,
    keys: &[Key],
    key: &str,
) -> Option<&'a mut Value> {
    match table.get_mut(segment) {
        Some(item) => get_value_from_item(item, segment, keys, key),
        None => {
            println!("ERROR: Key segment '{segment}' is missing");
            None
        }
    }
}
