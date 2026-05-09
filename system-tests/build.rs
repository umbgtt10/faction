// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::configure()
        .out_dir("src/transport/grpc/generated")
        .compile_protos(
            &["src/transport/grpc/proto/faction.proto"],
            &["src/transport/grpc/proto"],
        )?;
    Ok(())
}
