// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde_json::Value;

const RESULTS: [&str; 3] = ["Accepted", "Rejected", "Probed"];

// The growth rule (§1, §17): a vector released at an earlier version is
// answered alike by every later one, but for the commands a later version
// added, which its receipts' admissible lists do not name. A vector not yet
// released is answered whole.
pub struct Growth {
    released: Value,
}

impl Growth {
    pub fn new(released: Value) -> Self {
        Self { released }
    }

    // A receipt as the vector under `name` states it: its output's admissible
    // commands kept to those the version that released the vector names.
    pub fn as_released(&self, name: &str, receipt: &Value) -> Value {
        let Some(version) = self.version(name) else {
            return receipt.clone();
        };
        let named = self.named(&version);
        let mut kept = receipt.clone();
        for answer in RESULTS {
            if let Some(admissible) = kept["output"]
                .get_mut(answer)
                .and_then(|body| body.get_mut("admissible"))
                .and_then(Value::as_array_mut)
            {
                admissible.retain(|command| {
                    command
                        .as_str()
                        .is_some_and(|command| named.contains(&String::from(command)))
                });
            }
        }
        kept
    }

    // The commands a released version names: those released at it or before.
    fn named(&self, version: &[u64]) -> Vec<String> {
        Self::entries(&self.released["commands"])
            .filter(|entry| Self::parts(&entry["version"]).as_slice() <= version)
            .map(|entry| String::from(entry["name"].as_str().expect("a command's name")))
            .collect()
    }

    fn version(&self, name: &str) -> Option<Vec<u64>> {
        Self::entries(&self.released["vectors"])
            .find(|entry| entry["id"] == name)
            .map(|entry| Self::parts(&entry["version"]))
    }

    fn entries(list: &Value) -> impl Iterator<Item = &Value> {
        list.as_array().expect("a list of released.json").iter()
    }

    fn parts(version: &Value) -> Vec<u64> {
        version
            .as_str()
            .expect("a version")
            .split('.')
            .map(|part| part.parse().expect("a version's number"))
            .collect()
    }
}
