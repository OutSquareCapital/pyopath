# Contributing to PyOPath

Thank you for your interest in contributing to PyOPath! We welcome contributions from the community to help improve the library. This document outlines the guidelines and best practices for contributing.

## How to Contribute

The project is managed with uv for all tasks, including rust toolchain management.
Once modifications are done:

Run cargo clippy at the root

```bash
uv run cargo clippy
```

Re-install the package (uv run maturin won't update the binary in the venv)

```bash
uv sync --reinstall
```

Run tests.
Stubtester will run the doctests in the pyi file.

```bash
uv run pytest tests
uv run stubtester pyopath.pyi
```

Run benchmarks

```bash
uv run python scripts/benchmark.py
```

## Reference

the folder reference contains the vanilla python pathlib implementation, for a reference of how the library should behave and/or be implemented.
