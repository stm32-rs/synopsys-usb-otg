#!/usr/bin/env bash

set -euxo pipefail

cargo check --features "cortex-m fs"
cargo check --features "cortex-m hs"
cargo check --features "riscv fs"
