# Roadmap (full clone)

This is an execution plan to reach a full `pathlib` clone, with clear milestones and verification points.

## Milestone 0 — Project scaffold

Deliverables:

- maturin + PyO3 build pipeline
- import surface in Python package
- CI skeleton (optional)

Exit criteria:

- `uv run maturin develop` installs the extension
- `import pyopath` works

## Milestone 1 — PurePath (POSIX + Windows)

Deliverables:

- Complete parsing model for both flavors
- Implement full lexical API for `PurePath` family
- Correct `__fspath__`, `__str__`, `__repr__`, comparisons, hashing

Exit criteria:

- A dedicated PurePath test suite passes on Windows
- A Linux CI run passes (for POSIX flavor)

## Milestone 2 — Path filesystem operations

Deliverables:

- Core filesystem methods for `Path` family
- Correct error mapping to Python exceptions
- Platform-specific behavior matching CPython

Exit criteria:

- Filesystem tests pass on Windows and Linux

## Milestone 3 — Globbing

Deliverables:

- `.glob()` and `.rglob()` fully compatible
- Good performance on large directories

Exit criteria:

- CPython-derived glob tests pass

## Milestone 4 — Resolve / symlinks / canonicalization

Deliverables:

- `.resolve()` semantics matching CPython as closely as possible
- Documented differences if any

Exit criteria:

- Symlink resolution tests pass cross-platform

## Milestone 5 — CPython test parity

Deliverables:

- A large subset of CPython `Lib/test/test_pathlib.py` ported or reused
- Compatibility matrix updated to 95%+ parity

Exit criteria:

- Continuous test parity checks in CI

## Milestone 6 — Release

Deliverables:

- Wheels for Windows / Linux / macOS
- Versioning policy
- Changelog process

Exit criteria:

- `pip install` equivalent using wheel artifact works for end users (internal publishing method TBD)

## Tracking

For each milestone, maintain:

- checklist of methods
- test coverage mapping
- performance regressions benchmarks

## Links for reference

<https://pyo3.rs/v0.27.2/>
<https://docs.python.org/3/library/pathlib.html>
<https://doc.rust-lang.org/std/path/struct.Path.html>
