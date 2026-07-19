# P0-ADR-NoStdZeroUnsafe

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P0 (axiom)
- **Phase:** cross-cutting

## Context
The intended deployment spans bare-metal Cortex-M research clusters and cloud
processes — ideally the same source, the same crate.

## Decision
`#![no_std]` with `alloc`, and `#![deny(unsafe_code)]`. One crate runs from
microcontroller to cloud, with correctness resting on ownership rather than
unsafe.

## Forcing constraints / Evidence
`N/A` — a first-principles product decision. The target set makes `std`
unavailable on one end; the security posture makes `unsafe` undesirable
throughout (zero-unsafe clears procurement review by inspection).

## Rejected alternatives
`std` — richer, but excludes bare-metal. `unsafe` for micro-optimisation —
rejected: ownership already delivers the correctness, and any unsafe block
would forfeit the zero-unsafe guarantee.

## Consequences
State is `Box<dyn State>` plus `alloc` collections; no threads; `State: Send`.
No `std`-only types anywhere.

## Enforcement
`#![no_std]` and `#![deny(unsafe_code)]` at the crate root — compiler-enforced,
not review-enforced.
