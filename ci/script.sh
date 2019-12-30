#!/usr/bin/env bash

set -euxo pipefail

cargo check --features "stm32f429xx fs"
cargo check --features "stm32f429xx hs"
cargo check --features "stm32f401xx"
cargo check --features "gd32vf103xx"
