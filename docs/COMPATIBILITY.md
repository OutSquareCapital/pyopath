# Compatibility with `pathlib`

This document defines what “full clone” means for `pyopath`.

## Scope

Target is Python's standard library `pathlib` behavior:

- `PurePath`, `PurePosixPath`, `PureWindowsPath`
- `Path`, `PosixPath`, `WindowsPath`
- Methods and properties on these types
- `os.PathLike` protocol (`__fspath__`)

The goal is behavioral compatibility, not just API-shape compatibility.

## Key semantic areas to match

### 1) Flavor-specific parsing

Windows vs POSIX differences must match CPython:

- Drive letters (`C:`), roots (`\\`), anchors.
- UNC paths (`\\server\\share\\...`).
- Verbatim paths (`\\?\\...`).
- Separator normalization rules.

### 2) Lexical operations (PurePath)

Must match behaviors for:

- `.parts`, `.parent`, `.parents`
- `.name`, `.suffix`, `.suffixes`, `.stem`
- `.with_name()`, `.with_suffix()`
- `.joinpath()` and operator `/` behavior (Python side)
- `.relative_to()` and exceptions
- `.is_absolute()`

### 3) Normalization and resolution

Operations that consult the filesystem must match:

- `.resolve()` semantics (including symlink resolution behavior).
- Absolute path conversions.

Where CPython behavior differs by platform or version, we should:

- Match the current supported Python version behavior.
- Document the behavior if exact equivalence is not possible.

### 4) Filesystem operations (Path)

Must match:

- Existence and type checks: `.exists()`, `.is_file()`, `.is_dir()`, `.is_symlink()`, etc.
- Directory iteration: `.iterdir()`
- Reading/writing: `.read_text()`, `.read_bytes()`, `.write_text()`, `.write_bytes()`
- Creation/removal: `.mkdir()`, `.touch()`, `.unlink()`, `.rmdir()`
- Metadata: `.stat()`, `.lstat()`, `.owner()`, `.group()` (platform dependent)

### 5) Globbing

`pathlib` globbing semantics can be subtle.

Must match:

- `.glob()` and `.rglob()` patterns and edge-cases.
- Hidden files rules (platform-dependent).
- `**` behavior.

### 6) Comparison and hashing

Must match CPython rules:

- Comparisons are flavor-aware.
- Equality and ordering semantics.
- Hashing stability.

### 7) String representations

Must match:

- `str(path)`
- `repr(path)`
- `bytes(path)` where applicable
- `path.as_posix()`

### 8) Interop

`pyopath` should interoperate with:

- `os.fspath()`
- `os.PathLike`
- `pathlib.Path`

When accepting inputs, `pyopath` should accept any `os.PathLike`.

## Known hard areas

The following areas are historically tricky and require explicit attention:

- Windows normalization edge cases (UNC, verbatim, drive-relative vs absolute).
- Non-UTF8 paths on POSIX.
- Symlink resolution semantics.
- Globbing corner cases.

These areas should have dedicated test coverage.
