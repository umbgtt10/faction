# Copyright (c) 2025-2026 Umberto Gotti
# SPDX-License-Identifier: MIT

# just looks for a POSIX `sh` on Windows and there is not reliably one on PATH:
# Git for Windows ships sh.exe without putting it there, and a resolvable
# bash.exe may belong to WSL, a separate toolchain. PowerShell is the one shell
# guaranteed present. Only recipe bodies are shell-interpreted, so this affects
# Windows alone; the Linux and macOS runners use just's default `sh`.
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# Set by just itself rather than per recipe line, so it reaches whichever shell
# runs the body without needing sh and PowerShell syntax for the same thing.
export RUSTFLAGS := "-D warnings"

# CI fails on drift instead of silently rewriting files nobody is there to
# review; a local run still formats in place.
fmt_mode := if env('CI', '') != '' { '--check' } else { '' }

default:
    @just --list

# Every commit message carries the library's grip and braintax, written in by
# .githooks/commit-msg. Git runs a repository's own hooks only where a clone
# asks it to: run this once a clone, on each machine.
hooks:
    git config core.hooksPath .githooks

# Formatting, clippy, the no_std check, the tests, then the system tests --
# each in a process of its own with ports of its own, run in parallel by
# slotgate. The no_std check builds the five no_std crates for a target with no
# std at all, which a host build cannot prove.
stage1:
    cargo fmt {{fmt_mode}}
    cargo clippy --workspace --all-targets -- -D warnings
    cargo check -p faction -p faction-core-validation -p faction-protocol -p faction-protocol-validation -p faction-spec-model --no-default-features --lib --target thumbv7em-none-eabihf
    cargo test -p faction -p faction-chaos -p faction-core-validation -p faction-ladder -p faction-memory -p faction-protocol -p faction-protocol-validation -p faction-spec-model -p xtask
    cargo test -p faction-system-tests --lib --bins
    cargo xtask system-tests

# The chaos validation once: the deterministic engine's single run of seeds
# and every regression recorded, as stage 1 runs them; then the real engine's
# list -- three seeds in each of the six thread and process cases, on the
# machine's clock -- its logs in logs/chaos/<run>/, the newest 50 runs kept.
chaos:
    cargo test -q -p faction-chaos
    cargo xtask chaos real

# Chaos seeds played on the deterministic engine, one after another: for an
# hour, or as long as CHAOS_DURATION says (seconds, or with s, m or h), or for
# CHAOS_SCENARIOS scenarios, from a fresh seed or from CHAOS_SEED. Each failure
# is replayed with its logs, shrunk and recorded in
# validation/chaos/chaos-regressions.json; the run's logs go to
# logs/chaos/<run>/, the newest 50 runs kept.
chaos-soak:
    cargo xtask chaos soak

# One chaos seed replayed on the deterministic engine, every node's log
# written under logs/chaos/<run>/<seed>/ with their consolidation, the newest
# 50 runs kept: the run a failure reported, to be read.
chaos-replay seed:
    cargo xtask chaos replay {{seed}}

# House rules, duplication, CRAP, coverage, mirrored tests, file risk and
# dependencies, in that order: see xtask/src/main.rs.
stage2:
    cargo xtask stage2

# The upgrades the workspace could take, each judged by cargo-ripe4rust ripe,
# worth asking about, or not yet, against the gates Cargo.toml declares for it:
# stage1, then stage2. It prints the plan and applies nothing.
ripe:
    cargo ripe4rust plan
