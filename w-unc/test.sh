#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-../target}"
set -e

cd "$(dirname $0)"

UNC_ENABLE_SANDBOX_LOG=1
cargo run --example ft_wunc
