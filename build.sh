#!/usr/bin/env bash

cargo run --bin stub_gen --features stub_gen
./.venv/bin/maturin dev --release
