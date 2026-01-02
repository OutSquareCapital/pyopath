# Development guide

This project uses:

- `uv` for Python environments and dependencies
- Rust toolchain (`cargo`) for building
- `maturin` for packaging the PyO3 extension

## Local setup (Windows)

1) Sync the `uv` environment:

```bash
+uv sync
```

1) Install maturin (choose one):

```bash
uv tool install maturin
```

or

```bash
cargo install maturin
```

1) Develop-install the extension:

```bash
uv run maturin develop
```

## Testing

Testing strategy:

- Unit tests for lexical behavior (PurePath) independent of filesystem.
- Filesystem tests using temporary directories.
- Cross-platform test runs (Windows + Linux minimum).

Suggested tooling:

- `pytest`

Suggested commands:

```bash
uv run pytest
```

## Versioning

Recommended:

- Semantic versioning.
- Keep parity tracking with the Python version(s) supported.

## Release

Build wheels:

```bash
uv run maturin build
```

If publishing to an internal index or GitHub Releases, document the release path in this file.

## CI

Minimum CI matrix:

- Windows latest
- Ubuntu latest

Optional:

- macOS

CI should:

- build wheels
- run tests against built artifacts
- optionally run a parity subset of CPython tests
