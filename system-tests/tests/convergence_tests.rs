// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rstest::rstest;

#[derive(Debug, Clone, Copy)]
enum Spawn {
    Task,
    Thread,
    Process,
}

#[derive(Debug, Clone, Copy)]
enum Transport {
    InMemory,
    Channels,
    Tcp,
    Grpc,
}

#[rstest]
#[case::task_in_memory(Spawn::Task, Transport::InMemory)]
fn cluster_reaches_bootstrapped(#[case] _spawn: Spawn, #[case] _transport: Transport) {}
