"""Type stubs for pyopath - a pathlib clone implemented in Rust."""

from collections.abc import Sequence
from typing import IO, Self

__version__: str
__all__: list[str] = [
    "Path",
    "PosixPath",
    "PurePath",
    "PurePosixPath",
    "PureWindowsPath",
    "WindowsPath",
]

class PurePath:
    r"""A generic class that represents the system's path flavour.

    Instantiating it creates either a `PurePosixPath` or a `PureWindowsPath`):
    ```python
    >>> from pyopath import PurePath, PurePosixPath, PureWindowsPath
    >>> PurePath('setup.py')      # Running on a Unix machine
    PurePosixPath('setup.py')

    ```
    Each element of pathsegments can be either:
    - a string representing a path segment
    - an object implementing the `os.PathLike` interface where the `__fspath__()` method returns a string, such as another path object:
    ```python
    >>> PurePath('foo', 'some/path', 'bar')
    PurePosixPath('foo/some/path/bar')
    >>> PurePath(Path('foo'), Path('bar'))
    PurePosixPath('foo/bar')

    ```
    When pathsegments is empty, the current directory is assumed:
    ```python
    >>> PurePath()
    PurePosixPath(".")

    ```
    If a segment is an absolute path, all previous segments are ignored (like os.path.join()):
    ```python
    >>> PurePath("/etc", "/usr", "lib64")
    PurePosixPath("/usr/lib64")
    >>> PureWindowsPath("c:/Windows", "d:bar")
    PureWindowsPath("d:bar")

    ```
    On Windows, the drive is not reset when a rooted relative path segment (e.g., r'\foo') is encountered:
    ```python
    >>> PureWindowsPath("c:/Windows", "/Program Files")
    PureWindowsPath("c:/Program Files")

    ```
    Spurious slashes and single dots are collapsed, but double dots ('..') and leading double slashes ('//') are not, since this would change the meaning of a path for various reasons
    (e.g. symbolic links, UNC paths):
    ```python
    >>> PurePath("foo//bar")
    PurePosixPath("foo/bar")
    >>> PurePath("//foo/bar")
    PurePosixPath("//foo/bar")
    >>> PurePath("foo/./bar")
    PurePosixPath("foo/bar")
    >>> PurePath("foo/../bar")
    PurePosixPath("foo/../bar")

    ```
    (a naïve approach would make `PurePosixPath('foo/../bar')` equivalent to `PurePosixPath('bar')`, which is wrong if foo is a symbolic link to another directory)

    Pure path objects implement the `os.PathLike` interface, allowing them to be used anywhere the interface is accepted.
    """

    def __new__(cls, *args: str | PurePath) -> Self: ...
    @property
    def drive(self) -> str:
        r"""A string representing the drive letter or name, if any.

        Returns:
            str: The drive component of the path.

        Examples:
        ```python
        >>> PureWindowsPath("c:/Program Files/").drive
        "c:"
        >>> PureWindowsPath("/Program Files/").drive
        ""
        >>> PurePosixPath("/etc").drive
        ""

        ```
        UNC shares are also considered drives:
        ```python
        >>> PureWindowsPath("//host/share/foo.txt").drive
        "\\\\host\\share"

        ```
        """
    @property
    def root(self) -> str: ...
    @property
    def anchor(self) -> str: ...
    @property
    def parts(self) -> Sequence[str]: ...
    @property
    def name(self) -> str: ...
    @property
    def suffix(self) -> str: ...
    @property
    def suffixes(self) -> Sequence[str]: ...
    @property
    def stem(self) -> str: ...
    @property
    def parent(self) -> Self: ...
    @property
    def parents(self) -> Sequence[Self]: ...

    # Methods
    def is_absolute(self) -> bool: ...
    def is_relative_to(self, other: Self | str) -> bool: ...
    def relative_to(self, other: Self | str, *, walk_up: bool = False) -> Self: ...
    def joinpath(self, *args: str | Self) -> Self: ...
    def with_name(self, name: str) -> Self: ...
    def with_stem(self, stem: str) -> Self: ...
    def with_suffix(self, suffix: str) -> Self: ...
    def as_posix(self) -> str: ...

    # Dunder methods
    def __fspath__(self) -> str: ...
    def __truediv__(self, other: str | Self) -> Self: ...
    def __rtruediv__(self, other: str) -> Self: ...
    def __hash__(self) -> int: ...
    def __eq__(self, other: object) -> bool: ...
    def __ne__(self, other: object) -> bool: ...
    def __lt__(self, other: Self) -> bool: ...
    def __le__(self, other: Self) -> bool: ...
    def __gt__(self, other: Self) -> bool: ...
    def __ge__(self, other: Self) -> bool: ...

class PurePosixPath(PurePath):
    """A subclass of `PurePath`, this path flavour represents non-Windows filesystem paths.

    ```python
    >>> from pyopath import PurePosixPath
    >>> PurePosixPath('/etc/hosts')
    PurePosixPath('/etc/hosts')

    ```
    pathsegments is specified similarly to PurePath.
    """

    def __new__(cls, *args: str | PurePath) -> Self: ...

class PureWindowsPath(PurePath):
    """A subclass of `PurePath`, this path flavour represents Windows filesystem paths, including UNC paths.

    ```python
    >>> from pyopath import PureWindowsPath
    >>> PureWindowsPath('c:/', 'Users', 'Ximénez')
    PureWindowsPath('c:/Users/Ximénez')
    >>> PureWindowsPath('//server/share/file')
    PureWindowsPath('//server/share/file')

    ```
    pathsegments is specified similarly to PurePath.
    """

    def __new__(cls, *args: str | PurePath) -> Self: ...

class StatResult:
    """Result of a stat() call."""

    @property
    def st_mode(self) -> int: ...
    @property
    def st_size(self) -> int: ...
    @property
    def st_mtime(self) -> float: ...
    @property
    def st_atime(self) -> float: ...
    @property
    def st_ctime(self) -> float: ...

class Path(PurePath):
    """A concrete path that provides filesystem operations."""

    def __new__(cls, *args: str | PurePath) -> Self: ...

    # Filesystem query methods
    def exists(self) -> bool: ...
    def is_file(self) -> bool: ...
    def is_dir(self) -> bool: ...
    def is_symlink(self) -> bool: ...
    def stat(self) -> StatResult: ...
    def lstat(self) -> StatResult: ...

    # Path resolution
    def absolute(self) -> Self: ...
    def resolve(self, strict: bool = False) -> Self: ...
    def readlink(self) -> Self: ...

    # Directory operations
    def mkdir(
        self, mode: int = 0o777, parents: bool = False, exist_ok: bool = False
    ) -> None: ...
    def rmdir(self) -> None: ...
    def iterdir(self) -> Sequence[Self]: ...
    def glob(self, pattern: str) -> Sequence[Self]: ...
    def rglob(self, pattern: str) -> Sequence[Self]: ...

    # File operations
    def touch(self, exist_ok: bool = True) -> None: ...
    def unlink(self, missing_ok: bool = False) -> None: ...
    def rename(self, target: Self | str) -> Self: ...
    def replace(self, target: Self | str) -> Self: ...

    # Read/write operations
    def read_text(self, encoding: str | None = None) -> str: ...
    def write_text(self, data: str, encoding: str | None = None) -> int: ...
    def read_bytes(self) -> bytes: ...
    def write_bytes(self, data: bytes) -> int: ...

    # File opening
    def open(
        self,
        mode: str = "r",
        buffering: int = -1,
        encoding: str | None = None,
        errors: str | None = None,
        newline: str | None = None,
    ) -> IO[str]: ...

    # Static methods
    @staticmethod
    def cwd() -> Path: ...
    @staticmethod
    def home() -> Path: ...

class PosixPath(Path, PurePosixPath):
    """A POSIX concrete path with filesystem operations."""

    def __new__(cls, *args: str | PurePath) -> Self: ...
    @staticmethod
    def cwd() -> PosixPath: ...
    @staticmethod
    def home() -> PosixPath: ...

class WindowsPath(Path, PureWindowsPath):
    """A Windows concrete path with filesystem operations."""

    def __new__(cls, *args: str | PurePath) -> Self: ...
    @staticmethod
    def cwd() -> WindowsPath: ...
    @staticmethod
    def home() -> WindowsPath: ...
