// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use boon::Compiler;
use boon::SchemaIndex;
use boon::Schemas;
use serde_json::Value;

// The specification's data model, compiled once: its root is a vector, and the
// matrix, a configuration and a step -- a log's first line and every line after
// it -- have definitions of their own in it.
pub struct SpecSchema {
    schemas: Schemas,
    configuration: SchemaIndex,
    vector: SchemaIndex,
    matrix: SchemaIndex,
    step: SchemaIndex,
}

impl SpecSchema {
    pub fn new(schema: Value) -> Self {
        let id = String::from(schema["$id"].as_str().expect("the schema's id"));
        let mut compiler = Compiler::new();
        compiler
            .add_resource(&id, schema)
            .expect("the schema loads");
        let mut schemas = Schemas::new();
        let vector = compiler
            .compile(&id, &mut schemas)
            .expect("a vector's definition compiles");
        let configuration = compiler
            .compile(&format!("{id}#/$defs/configuration"), &mut schemas)
            .expect("a configuration's definition compiles");
        let matrix = compiler
            .compile(&format!("{id}#/$defs/matrix"), &mut schemas)
            .expect("the matrix's definition compiles");
        let step = compiler
            .compile(&format!("{id}#/$defs/step"), &mut schemas)
            .expect("a step's definition compiles");
        Self {
            schemas,
            configuration,
            vector,
            matrix,
            step,
        }
    }

    pub fn admits_configuration(&self, configuration: &Value) -> bool {
        self.schemas
            .validate(configuration, self.configuration)
            .is_ok()
    }

    pub fn admits_matrix(&self, matrix: &Value) -> bool {
        self.schemas.validate(matrix, self.matrix).is_ok()
    }

    pub fn admits_step(&self, step: &Value) -> bool {
        self.schemas.validate(step, self.step).is_ok()
    }

    pub fn admits_vector(&self, vector: &Value) -> bool {
        self.schemas.validate(vector, self.vector).is_ok()
    }
}
