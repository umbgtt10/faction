// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The specification as files: the matrix and every vector valid against the
// schema, the released vectors valid against it too, and every released
// vector still there, byte for byte -- a released vector is never removed or
// edited; and no word the vocabulary leaves out among the schema's names or
// the crate's public ones, as cartel's are checked.

use crate::support::spec_files::SpecFiles;
use crate::support::spec_schema::SpecSchema;
use serde_json::Value;
use serde_json::from_slice;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Write;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

// The words faction's vocabulary leaves out (ARCHITECTURE.md): cartel's --
// each one protocol's, a role, a phase's name, what is decided, a rule or
// evidence -- but four of faction's own: its cluster `view`, the layer above
// that `commit`s a change and reports it `committed`, and the `epoch`s Phase 5
// brings.
const LEFT_OUT: [&str; 22] = [
    "term",
    "height",
    "slot",
    "leader",
    "candidate",
    "proposer",
    "validator",
    "prepare",
    "prevote",
    "block",
    "log",
    "entry",
    "index",
    "majority",
    "supermajority",
    "certificate",
    "signature",
    "proof",
    "elected",
    "prepared",
    "decided",
    "value",
];
const PUBLIC_ITEMS: [&str; 7] = [
    "pub struct ",
    "pub enum ",
    "pub fn ",
    "pub mod ",
    "pub trait ",
    "pub type ",
    "pub const ",
];

// The names the crate declares in public, from its source: its public items,
// the fields of its records and the values of its enumerations, each split
// into its words.
fn declared(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut in_enumeration = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        if in_enumeration && line == "}" {
            in_enumeration = false;
        } else if in_enumeration && line.starts_with("    ") && !line.starts_with("        ") {
            found.extend(words(identifier(trimmed)));
        }
        for item in PUBLIC_ITEMS {
            if let Some(rest) = trimmed.strip_prefix(item) {
                found.extend(words(identifier(rest)));
                in_enumeration |= item == "pub enum ";
            }
        }
        if let Some((field, _)) = trimmed
            .strip_prefix("pub ")
            .and_then(|rest| rest.split_once(':'))
            && field
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_')
        {
            found.extend(words(field));
        }
    }
    found
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut written, byte| {
            write!(written, "{byte:02x}").expect("writing to a string");
            written
        })
}

// Every Rust source file under a folder, its subfolders included.
// The form of a released vector whose digest is `expected`: kept under
// released/ once a re-encoding replaced it, in force under vectors/ otherwise.
fn form(id: &str, expected: &str) -> Option<Vec<u8>> {
    SpecFiles::released_bytes(id)
        .into_iter()
        .chain(SpecFiles::vector_bytes(id))
        .find(|bytes| digest(bytes) == expected)
}

fn identifier(text: &str) -> &str {
    let end = text
        .find(|character: char| !(character.is_alphanumeric() || character == '_'))
        .unwrap_or(text.len());
    &text[..end]
}

// A vector as 0.9.0 writes it, from the form 0.8.0 wrote: every view's change
// in flight a list, `None` the empty one (§1).
fn listed_form(written: &Value) -> Value {
    match written {
        Value::Object(members) => {
            let is_view = members.contains_key("peer_state");
            Value::Object(
                members
                    .iter()
                    .map(|(key, value)| match (is_view, key.as_str()) {
                        (true, "reconfiguration") if *value == json!({ "None": {} }) => {
                            (String::from("reconfigurations"), json!([]))
                        }
                        (true, "reconfiguration") => {
                            (String::from("reconfigurations"), json!([value]))
                        }
                        _ => (key.clone(), listed_form(value)),
                    })
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(listed_form).collect()),
        other => other.clone(),
    }
}

// Every name the schema gives a definition, a member or a value of an
// enumeration, split into its words: `next_receipt` and `QuorumReached` into
// two each.
fn names(value: &Value, found: &mut Vec<String>) {
    if let Some(object) = value.as_object() {
        for (key, inner) in object {
            if !key.starts_with('$') {
                found.extend(words(key));
            }
            if key == "required" {
                for name in inner
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    found.extend(words(name));
                }
            }
            names(inner, found);
        }
    } else if let Some(items) = value.as_array() {
        for item in items {
            names(item, found);
        }
    }
}

// A vector as 0.8.0 writes it, from its released form: each step's result its
// receipt's output, numbered by its position; the closing answer the closing
// Probe's receipt; the configuration's peer id its instance's node, at
// incarnation 1 (§1).
fn reencoded_form(released: &Value) -> Value {
    let mut configuration = released["configuration"].clone();
    let node = configuration
        .as_object_mut()
        .and_then(|members| members.remove("peer_id"))
        .expect("a released configuration's peer id");
    configuration["instance"] = json!({ "node": node, "incarnation": "1" });
    let steps: Vec<Value> = released["steps"]
        .as_array()
        .expect("a vector's steps")
        .iter()
        .enumerate()
        .map(|(index, step)| {
            json!({
                "command": step["command"],
                "receipt": { "number": (index + 1).to_string(), "output": step["result"] },
            })
        })
        .collect();
    let closing = (steps.len() + 1).to_string();
    json!({
        "description": released["description"],
        "rows": released["rows"],
        "configuration": configuration,
        "steps": steps,
        "final": { "number": closing, "output": { "Probed": released["final"] } },
    })
}

// What the re-encoding of `version` writes of a vector's form before it.
fn reencoding(version: &str, before: &Value) -> Value {
    match version {
        "0.8.0" => reencoded_form(before),
        "0.9.0" => listed_form(before),
        other => panic!("no re-encoding at {other}"),
    }
}

fn sources(folder: &Path) -> Vec<PathBuf> {
    read_dir(folder)
        .expect("a source folder")
        .map(|entry| entry.expect("an entry").path())
        .flat_map(|path| {
            if path.is_dir() {
                sources(&path)
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                vec![path]
            } else {
                Vec::new()
            }
        })
        .collect()
}

// A name's words, lowercase: split at each underscore and where a capital
// follows a lowercase letter or a digit, so `next_receipt`, `QuorumReached`
// and `BLOCK_HEIGHT` each give their two.
fn words(name: &str) -> Vec<String> {
    let mut split = Vec::new();
    let mut current = String::new();
    let mut after_lowercase = false;
    for character in name.chars() {
        let boundary = character == '_' || (character.is_uppercase() && after_lowercase);
        if boundary && !current.is_empty() {
            split.push(current.clone());
            current.clear();
        }
        after_lowercase = character.is_lowercase() || character.is_ascii_digit();
        if character != '_' {
            current.extend(character.to_lowercase());
        }
    }
    if !current.is_empty() {
        split.push(current);
    }
    split
}

#[test]
fn admits_matrix_of_the_specification_holds() {
    // Arrange
    let schema = SpecSchema::new(SpecFiles::schema());

    // Act
    let admitted = schema.admits_matrix(&SpecFiles::matrix());

    // Assert
    assert!(
        admitted,
        "matrix.json is not valid against the schema's matrix"
    );
}

#[test]
fn admits_released_of_the_specification_holds() {
    // Arrange
    let schema = SpecSchema::new(SpecFiles::schema());

    // Act & Assert
    assert!(schema.admits_released(&SpecFiles::released()));
}

#[test]
fn admits_vector_of_every_vector_holds() {
    // Arrange
    let schema = SpecSchema::new(SpecFiles::schema());

    // Act
    let refused: Vec<String> = SpecFiles::vectors()
        .into_iter()
        .filter(|(_, vector)| !schema.admits_vector(vector))
        .map(|(name, _)| name)
        .collect();

    // Assert
    assert!(
        refused.is_empty(),
        "vectors not valid against the schema: {refused:?}"
    );
}

#[test]
fn declared_of_the_crate_holds_no_word_the_vocabulary_leaves_out() {
    // Arrange
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/core/src");

    // Act
    let found: Vec<String> = sources(&source)
        .into_iter()
        .map(|path| read_to_string(path).expect("a source file's text"))
        .flat_map(|text| declared(&text))
        .collect();

    // Assert
    let left_out: Vec<&String> = found
        .iter()
        .filter(|word| LEFT_OUT.contains(&word.as_str()))
        .collect();
    assert!(found.contains(&String::from("peer")));
    assert!(left_out.is_empty(), "{left_out:#?}");
}

#[test]
fn names_of_the_schema_hold_no_word_the_vocabulary_leaves_out() {
    // Arrange
    let mut found = Vec::new();

    // Act
    names(&SpecFiles::schema(), &mut found);

    // Assert
    let left_out: Vec<&String> = found
        .iter()
        .filter(|word| LEFT_OUT.contains(&word.as_str()))
        .collect();
    assert!(!found.is_empty());
    assert!(left_out.is_empty(), "{left_out:#?}");
}

#[test]
fn vector_bytes_of_every_released_vector_are_as_released() {
    // Arrange
    let released = SpecFiles::released();

    // Act
    let departures: Vec<String> = released["vectors"]
        .as_array()
        .expect("the released vectors")
        .iter()
        .filter_map(|entry| {
            let id = entry["id"].as_str().expect("a released vector's id");
            let expected = entry["sha256"].as_str().expect("its digest");
            let reencodings: Vec<&str> = entry
                .get("reencoded")
                .and_then(Value::as_array)
                .map(|list| {
                    list.iter()
                        .map(|reencoding| {
                            reencoding["sha256"].as_str().expect("a re-encoded digest")
                        })
                        .collect()
                })
                .unwrap_or_default();
            let bytes = match (entry.get("withdrawn"), reencodings.last()) {
                (Some(_), _) => SpecFiles::withdrawn_bytes(id),
                (None, Some(_)) => form(id, expected),
                (None, None) => SpecFiles::vector_bytes(id),
            };
            let in_force = reencodings.last().is_none_or(|last| {
                SpecFiles::vector_bytes(id).is_some_and(|bytes| digest(&bytes) == *last)
            });
            let replaced_kept = reencodings
                .iter()
                .rev()
                .skip(1)
                .all(|replaced| form(id, replaced).is_some());
            match bytes {
                None => Some(format!("{id}: released, and missing")),
                Some(bytes) if digest(&bytes) != expected => {
                    Some(format!("{id}: released, and changed"))
                }
                Some(_) if !in_force => Some(format!("{id}: re-encoded, and changed")),
                Some(_) if !replaced_kept => Some(format!("{id}: re-encoded, and a form lost")),
                Some(_) => None,
            }
        })
        .collect();

    // Assert
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn vectors_of_the_specification_hold_no_vector_withdrawn() {
    // Arrange
    let released = SpecFiles::released();
    let in_force: Vec<String> = SpecFiles::vectors()
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    // Act
    let withdrawn: Vec<&str> = released["vectors"]
        .as_array()
        .expect("the released vectors")
        .iter()
        .filter(|entry| entry.get("withdrawn").is_some())
        .map(|entry| entry["id"].as_str().expect("a released vector's id"))
        .collect();

    // Assert
    assert_eq!(
        withdrawn,
        ["113-provisional-acknowledgement-from-behind-and-removal"]
    );
    for id in withdrawn {
        assert!(!in_force.iter().any(|name| name == id), "{id}");
        assert!(SpecFiles::withdrawn_bytes(id).is_some(), "{id}");
    }
}

#[test]
fn vectors_of_the_specification_re_encoded_answer_as_their_released_bytes() {
    // Arrange
    let released = SpecFiles::released();
    let reencoded: Vec<&Value> = released["vectors"]
        .as_array()
        .expect("the released vectors")
        .iter()
        .filter(|entry| entry.get("reencoded").is_some())
        .collect();

    // Act
    let departures: Vec<String> = reencoded
        .iter()
        .flat_map(|entry| {
            let id = entry["id"].as_str().expect("a released vector's id");
            let mut digests = vec![entry["sha256"].as_str().expect("its digest")];
            let mut versions = Vec::new();
            for reencoding in entry["reencoded"].as_array().expect("its re-encodings") {
                digests.push(reencoding["sha256"].as_str().expect("a re-encoded digest"));
                versions.push(
                    reencoding["version"]
                        .as_str()
                        .expect("a re-encoding's version"),
                );
            }
            versions
                .into_iter()
                .zip(digests.windows(2))
                .filter_map(|(version, pair)| {
                    let before: Value = from_slice(&form(id, pair[0])?).ok()?;
                    let after: Value = from_slice(&form(id, pair[1])?).ok()?;
                    (after != reencoding(version, &before))
                        .then(|| format!("{id}: re-encoded at {version} otherwise"))
                })
                .collect::<Vec<String>>()
        })
        .collect();

    // Assert
    assert_eq!(reencoded.len(), 115);
    assert!(departures.is_empty(), "{departures:#?}");
}
