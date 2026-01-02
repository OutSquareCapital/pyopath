# pyopath

`pyopath` is a **full-compatibility clone of Python's `pathlib`** implemented in **Rust** via **PyO3**.

Drop-in replacement for `pathlib` with better performance.

## Installation

```bash
uv add pyopath
```

## Usage

```python
from pyopath import Path, PurePath

# Exactement comme pathlib
p = Path("src/lib.rs")
print(p.exists())        # True
print(p.read_text()[:50])  # "//! pyopath..."
print(p.suffix)          # ".rs"
print(p.parent)          # src

# Globbing
for f in Path(".").glob("**/*.rs"):
    print(f)

# Filesystem operations
p = Path("test.txt")
p.write_text("hello")
p.unlink()
```

## API

### Classes

| Pure (lexical only) | Concrete (filesystem) |
|---------------------|----------------------|
| `PurePath` | `Path` |
| `PurePosixPath` | `PosixPath` |
| `PureWindowsPath` | `WindowsPath` |

### Properties

`drive`, `root`, `anchor`, `parts`, `name`, `suffix`, `suffixes`, `stem`, `parent`, `parents`

### Methods

| Lexical | Filesystem |
|---------|------------|
| `is_absolute()` | `exists()`, `is_file()`, `is_dir()`, `is_symlink()` |
| `is_relative_to()` | `stat()`, `lstat()` |
| `relative_to()` | `absolute()`, `resolve()`, `readlink()` |
| `joinpath()`, `/` operator | `mkdir()`, `rmdir()`, `iterdir()` |
| `with_name()`, `with_stem()`, `with_suffix()` | `glob()`, `rglob()` |
| `as_posix()` | `touch()`, `unlink()`, `rename()`, `replace()` |
| | `read_text()`, `write_text()`, `read_bytes()`, `write_bytes()` |
| | `open()`, `cwd()`, `home()` |

## What's missing

- `match()` / `full_match()` - glob pattern matching
- `walk()` - recursive directory traversal (Python 3.12+)
- `owner()` / `group()` - Unix-only metadata
- `is_mount()`, `is_block_device()`, etc. - special Unix checks

## Development

```bash
uv sync --reinstall  # Build and install
uv run pytest        # Run tests
```

## Links

- [pathlib documentation](https://docs.python.org/3/library/pathlib.html)
- [PyO3](https://pyo3.rs/)
