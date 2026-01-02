# Architecture

## Goals

- Provide a full-compatibility clone of Python's standard library `pathlib`.
- Keep Python-facing classes and behaviors as close as practical to CPython.
- Implement the core in Rust for performance and strong invariants.
- Expose the Rust implementation to Python via PyO3, packaged with maturin.

## High-level design

The project is split into two layers:

1) **Rust core**: the semantics and implementation.
2) **Python façade**: a thin public API surface matching `pathlib` names, docstrings, and Python behaviors.

The key rule is: *Python code should be a compatibility layer, not the implementation.*

## Modules

### Python package (`src/pyopath`)

Responsibilities:

- Provide import surface mirroring `pathlib` (e.g. `Path`, `PurePath`, etc.).
- Maintain Python ergonomics: `__fspath__`, `__str__`, `__repr__`, rich comparisons, iteration protocols.
- Provide any compatibility glue that is easier/safer in Python than in Rust.

Non-responsibilities:

- Heavy path parsing, normalization, filesystem walking, globbing.

### Rust extension (planned)

A single PyO3 extension module (suggested name `pyopath._pyopath`) exposes:

- Internal Rust-backed path objects.
- Conversion helpers for `os.PathLike` and `pathlib.Path`.
- Fast implementations of heavy operations.

## Type model

`pathlib` has two conceptual families:

- **Pure paths**: lexical operations only (no filesystem access).
- **Concrete paths**: filesystem operations.

We mirror this with:

- `PurePath` / `PurePosixPath` / `PureWindowsPath`
- `Path` / `PosixPath` / `WindowsPath`

Additionally, each instance is tagged with a **flavor**:

- POSIX flavor
- Windows flavor

Flavor is required because many semantics differ (parsing, anchors, drive letters, UNC paths, etc.).

## Boundary decisions

### Encoding and non-UTF8 paths

Rust `std::path` uses `OsStr`/`OsString` which are not necessarily UTF-8. This is essential to correctly support non-UTF8 paths.

The Rust core MUST:

- Store paths as `OsString` / `PathBuf`.
- Avoid forcing UTF-8 conversions except where required by Python (`str`).
- Preserve roundtrippability where possible.

### Filesystem operations

Filesystem operations should be implemented in Rust using `std::fs` and carefully chosen crates when required for compatibility.

The Python layer should only:

- Translate errors to Python exceptions.
- Apply compatibility behaviors that depend on CPython-level details.

## Error strategy

Errors must surface as the appropriate Python exceptions matching `pathlib` behavior:

- `ValueError` for invalid path operations (`relative_to` failures, invalid suffix operations, etc.).
- `TypeError` for incorrect argument types.
- `OSError` / subclasses for filesystem failures.

Rust code should preserve structured error info and map it at the boundary.

## Performance strategy

The largest performance wins typically come from:

- Iteration-heavy filesystem operations (`iterdir`, recursive walking).
- Globbing / matching.
- Avoiding repeated parsing of the same path.

We should keep the Rust core representation canonical and avoid extra allocations.

## Compatibility contract

The compatibility contract is defined in [COMPATIBILITY.md](COMPATIBILITY.md).

Implementation work should be driven by:

- A compatibility checklist.
- A test suite derived from CPython `pathlib` tests (see roadmap).
