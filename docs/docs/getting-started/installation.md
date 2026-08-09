# Installation

## Requirements

- Python **3.10 or newer**
- `pip` (or your favorite Python package manager, such as `uv` or Poetry)

The runtime dependencies `orjson` and `watchdog` are installed automatically with the package.

## Install from PyPI

The package is published on [PyPI](https://pypi.org/project/oxapy/):

```bash
pip install oxapy
```

## Verify the installation

Make sure the import works:

```bash
python -c "from oxapy import Oxapy, Router, get; print('OxAPY installed')"
```

## Build from source

If you want the latest changes or plan to contribute, build the extension yourself. This requires a Rust toolchain and [uv](https://docs.astral.sh/uv/).

```bash
# Clone the repository
git clone https://github.com/j03-dev/oxapy
cd oxapy

# Install dependencies
uv sync

# Build the extension in development (editable) mode
uv run maturin dev --release

# Run the test suite
pytest -vv tests
```

Alternatively, `./build.sh` performs the development build for you.

## Next steps

Ready to write your first server? Head to the [Quickstart](./quickstart).
