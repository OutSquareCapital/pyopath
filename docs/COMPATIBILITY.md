# Compatibility with `pathlib`

## Implemented ✅

### Classes

- [x] `PurePath`, `PurePosixPath`, `PureWindowsPath`
- [x] `Path`, `PosixPath`, `WindowsPath`

### Properties

- [x] `drive`, `root`, `anchor`, `parts`
- [x] `name`, `suffix`, `suffixes`, `stem`
- [x] `parent`, `parents`

### Lexical methods

- [x] `is_absolute()`
- [x] `is_relative_to()`
- [x] `relative_to(walk_up=)`
- [x] `joinpath()` + `/` operator
- [x] `with_name()`, `with_stem()`, `with_suffix()`
- [x] `as_posix()`

### Filesystem methods

- [x] `exists()`, `is_file()`, `is_dir()`, `is_symlink()`
- [x] `stat()`, `lstat()`
- [x] `absolute()`, `resolve(strict=)`, `readlink()`
- [x] `mkdir(mode=, parents=, exist_ok=)`, `rmdir()`
- [x] `iterdir()`, `glob()`, `rglob()`
- [x] `touch(exist_ok=)`, `unlink(missing_ok=)`
- [x] `rename()`, `replace()`
- [x] `read_text(encoding=)`, `write_text(data, encoding=)`
- [x] `read_bytes()`, `write_bytes()`
- [x] `open(mode=, buffering=, encoding=, errors=, newline=)`
- [x] `cwd()`, `home()`

### Protocols

- [x] `__str__`, `__repr__`, `__fspath__`
- [x] `__truediv__`, `__rtruediv__`
- [x] `__hash__`, `__eq__`, `__ne__`
- [x] `__lt__`, `__le__`, `__gt__`, `__ge__`

## Missing ❌

### Pattern matching

- [ ] `match(pattern)` - glob-style pattern match
- [ ] `full_match(pattern)` - anchored pattern match

### Directory walking (Python 3.12+)

- [ ] `walk(top_down=, on_error=, follow_symlinks=)`

### Unix-specific

- [ ] `owner()`, `group()`
- [ ] `is_mount()`
- [ ] `is_block_device()`, `is_char_device()`, `is_fifo()`, `is_socket()`
- [ ] `chmod()`, `lchmod()`
- [ ] `symlink_to()`, `hardlink_to()`

### Other

- [ ] `samefile(other)`
- [ ] `expanduser()`
- [ ] `is_reserved()` (Windows)
