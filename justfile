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
# slotgate. The no_std check builds the four no_std crates for a target with no
# std at all, which a host build cannot prove.
stage1:
    cargo fmt {{fmt_mode}}
    cargo clippy --workspace --all-targets -- -D warnings
    cargo check -p faction -p faction-core-validation -p faction-protocol -p faction-protocol-validation --no-default-features --lib --target thumbv7em-none-eabihf
    cargo test -p faction -p faction-core-validation -p faction-ladder -p faction-protocol -p faction-protocol-validation -p xtask
    cargo test -p faction-system-tests --lib --bins
    cargo xtask system-tests

# House rules, duplication, CRAP, coverage, mirrored tests, file risk and
# dependencies, in that order: see xtask/src/main.rs.
stage2:
    cargo xtask stage2
