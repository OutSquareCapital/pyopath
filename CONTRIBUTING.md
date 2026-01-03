# Contributing to PyOPath

Thank you for your interest in contributing to PyOPath! We welcome contributions from the community to help improve the library. This document outlines the guidelines and best practices for contributing.

## How to Contribute

Run cargo clippy at the root

```bash
uv run cargo clippy
```

Re-install the package

```bash
uv sync --reinstall
```

Run tests

```bash
uv run pytest tests
```

Run benchmarks

```bash
uv run python scripts/benchmark.py
```
