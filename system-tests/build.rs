// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("src/transport/grpc/generated")?;
    tonic_build::configure()
        .out_dir("src/transport/grpc/generated")
        .compile_protos(
            &["src/transport/grpc/proto/faction.proto"],
            &["src/transport/grpc/proto"],
        )?;
    Ok(())
}
