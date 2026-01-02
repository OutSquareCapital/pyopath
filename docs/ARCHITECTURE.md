# Architecture

## Goals

- Provide a full-compatibility clone of Python's standard library `pathlib`.
- Keep Python-facing classes and behaviors as close as practical to CPython.
- Implement **everything** in Rust for performance and strong invariants.
- Expose the Rust implementation to Python via PyO3, packaged with maturin.

## High-level design

The project is **Pure Rust**:

- **Rust implementation**: all semantics, parsing, filesystem operations.
- **Python stubs (`.pyi`)**: type hints only, no runtime Python code.

The key rule is: *No Python code at runtime. Only Rust.*

## Modules

### Rust extension (`src/lib.rs`)

The single PyO3 extension module `pyopath` exposes:

- All path classes directly: `Path`, `PurePath`, `PosixPath`, `WindowsPath`, etc.
- Python ergonomics implemented in Rust: `__fspath__`, `__str__`, `__repr__`, rich comparisons, iteration protocols.
- Conversion helpers for `os.PathLike` and `pathlib.Path`.
- All filesystem operations.

### Type stubs (`pyopath.pyi`)

Responsibilities:

- Provide type hints for IDE support and static analysis.
- Mirror the public API surface.

Non-responsibilities:

- Any runtime behavior (stubs are not executed).

## Type model

`pathlib` has two conceptual families:

- **Pure paths**: lexical operations only (no filesystem access).
- **Concrete paths**: filesystem operations.

We mirror this in Rust with `#[pyclass]` structs:

- `PurePath` / `PurePosixPath` / `PureWindowsPath`
- `Path` / `PosixPath` / `WindowsPath`

Additionally, each instance is tagged with a **flavor**:

- POSIX flavor
- Windows flavor

Flavor is required because many semantics differ (parsing, anchors, drive letters, UNC paths, etc.).

All classes are implemented as Rust structs with PyO3 bindings.

## Boundary decisions

### Encoding and non-UTF8 paths

Rust `std::path` uses `OsStr`/`OsString` which are not necessarily UTF-8. This is essential to correctly support non-UTF8 paths.

The Rust core MUST:

- Store paths as `OsString` / `PathBuf`.
- Avoid forcing UTF-8 conversions except where required by Python (`str`).
- Preserve roundtrippability where possible.

### Filesystem operations

Filesystem operations are implemented in Rust using `std::fs` and carefully chosen crates when required for compatibility.

Error translation to Python exceptions is handled at the PyO3 boundary in Rust.

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
