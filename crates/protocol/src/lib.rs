// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod input_message;
pub mod message_translator;
pub mod output_message;
pub mod protocol;
pub mod result_view;
pub mod timer;
pub mod timer_event;
pub mod timer_message;
pub mod transport;
pub mod transport_message;
