## pyopath

`pyopath` is a **full-compatibility clone of Python's `pathlib`** whose implementation lives in **Rust**, exposed to Python via **PyO3** and built with **maturin**.

The goal is to provide a drop-in developer experience close to the standard library `pathlib`, while enabling:

- Higher performance for path operations and directory walking.
- Predictable cross-platform semantics (Windows / POSIX).
- A clean separation between **lexical path semantics** (PurePath) and **filesystem operations** (Path).

This repository currently contains the project skeleton and documentation. Implementation is intentionally staged and tracked in the roadmap.

### Non-goals

- Inventing a new API: compatibility and behavioral fidelity are prioritized.

## Status

Design and planning phase.

## Project layout

- `src/`: Rust source code (PyO3 extension module).
- `pyopath.pyi`: Python type stubs for IDE support.
- `docs/`: architecture, compatibility notes, and roadmap.

## API surface

Targeted public surface (mirrors `pathlib`):

- `PurePath`, `PurePosixPath`, `PureWindowsPath`
- `Path`, `PosixPath`, `WindowsPath`

As in `pathlib`, `Path` is the platform-specific concrete class returned by `Path(...)`.

## Development

### Prerequisites

- Windows supported (this repo is currently configured for Windows dev), with a Rust toolchain installed (`cargo`).
- `uv` available (used instead of `pip` and `python`).
- `maturin` available (can be installed via `uv` tooling or via `cargo`).

### Build / develop

During early development, the preferred workflow is an editable install into the `uv` environment:

```bash
uv run maturin develop --release
```

Build wheels:

```bash
uv run maturin build --release
```

Run tests:

```bash
uv run pytest
```

## Documentation

- See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the module design and boundaries.
- See [docs/COMPATIBILITY.md](docs/COMPATIBILITY.md) for the `pathlib` contract and tricky semantics.
- See [docs/ROADMAP.md](docs/ROADMAP.md) for the execution plan toward a full clone.
- See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for local dev, release, and CI guidance.
