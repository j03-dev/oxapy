#!/usr/bin/env bash

cargo run --bin stub_gen --features="stub-gen"
./.venv/bin/maturin dev --release
