// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::fs::create_dir_all;
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    create_dir_all("src/infra/transport/grpc/generated")?;
    configure()
        .out_dir("src/infra/transport/grpc/generated")
        .compile_protos(
            &["src/infra/transport/grpc/proto/faction.proto"],
            &["src/infra/transport/grpc/proto"],
        )?;
    Ok(())
}
