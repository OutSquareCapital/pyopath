import sys
import types
from collections.abc import Callable, Generator, Iterator, Sequence
from io import BufferedRandom, BufferedReader, BufferedWriter, FileIO, TextIOWrapper
from os import PathLike, stat_result
from pathlib.types import PathInfo
from typing import IO, Any, BinaryIO, ClassVar, Literal, Never, Self, overload

from _typeshed import (
    OpenBinaryMode,
    OpenBinaryModeReading,
    OpenBinaryModeUpdating,
    OpenBinaryModeWriting,
    OpenTextMode,
    ReadableBuffer,
    StrOrBytesPath,
    StrPath,
    Unused,
)

__all__ = [
    "Path",
    "PosixPath",
    "PurePath",
    "PurePosixPath",
    "PurePosixPath",
    "WindowsPath",
]

class PurePath(PathLike[str]):
    __slots__ = (
        "_drv",
        "_hash",
        "_parts_normcase_cached",
        "_raw_paths",
        "_root",
        "_st",
        "_str_normcase_cached",
        "_tail_cached",
    )
    parser: ClassVar[types.ModuleType]

    def full_match(
        self, pattern: StrPath, *, case_sensitive: bool | None = None
    ) -> bool:
        """Match this path against a glob-style pattern including wildcards.

        This method supports the full recursive wildcard `**`. Unlike `match()`,
        the pattern is matched against the entire path, not just the suffix.

        Args:
            pattern (str): A glob-style pattern using `*`, `?`, `[seq]`, and `**` wildcards.
            case_sensitive (bool | None): Override platform's case-sensitivity. If `None`, uses platform defaults.

        Returns:
            bool: `True` if the path matches the pattern, `False` otherwise.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('a/b.py').full_match('a/*.py')
        True
        >>> PurePosixPath('a/b/c.py').full_match('**/*.py')
        True
        >>> PurePosixPath('a/b.py').full_match('*.py')
        False

        ```
        """
    @property
    def parts(self) -> tuple[str, ...]:
        """Access the individual path components.

        Returns a tuple containing all path components, from the drive/root to
        the final component.

        The drive and root are regrouped into a single component on Windows.

        Returns:
            tuple[str, ...]: The path components.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/usr/bin/python3').parts
        ('/', 'usr', 'bin', 'python3')
        >>> PurePosixPath('Program Files/PSF').parts
        ('Program Files', 'PSF')

        ```
        """
    @property
    def drive(self) -> str:
        """The drive letter or name, if any.

        On Windows, this is the drive letter (e.g., 'c:') or UNC share path
        (e.g., '//host/share'). On POSIX systems, this is always an empty string.

        Returns:
            str: The drive component of the path, or an empty string if none.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/etc').drive
        ''

        ```
        """
    @property
    def root(self) -> str:
        """The root component of the path, if any.

        On POSIX systems, this is '/' for absolute paths and an empty string
        for relative paths. On Windows, this is '/' for absolute paths and
        an empty string for relative paths.

        Returns:
            str: The root component of the path, or an empty string if none.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('home/').root
        ''

        ```
        """
    @property
    def anchor(self) -> str:
        """The concatenation of the drive and root.

        This represents the filesystem root or reference point of the path.
        On Windows, this can include UNC paths. On POSIX systems, it's typically
        just the root '/' for absolute paths.

        Returns:
            str: The anchor (drive + root) of the path.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/etc').anchor
        '/'

        ```
        """
    @property
    def name(self) -> str:
        """The final path component, excluding drive and root.

        For a path like '/etc/passwd', this returns 'passwd'. For a directory
        like '/etc', this returns 'etc'. For UNC paths, the share name is not
        considered part of the name.

        Returns:
            str: The final path component, or empty string if the path is a root.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('my/library/setup.py').name
        'setup.py'

        ```
        """
    @property
    def suffix(self) -> str:
        """The last dot-separated portion of the final component.

        This is commonly called the file extension. Returns an empty string if
        the final component has no extension. Commonly referred to as the file
        extension.

        Returns:
            str: The suffix of the path (e.g., '.py'), or empty string if none.

        Examples:
        ```python
        >>> from pyopath import PurePath
        >>> PurePath('my/library/setup.py').suffix
        '.py'
        >>> PurePath('my/library.tar.gz').suffix
        '.gz'

        ```
        """
    @property
    def suffixes(self) -> list[str]:
        """List of all suffixes (file extensions) of the final component.

        Returns a list of all dot-separated suffixes. For a file like
        'archive.tar.gz', this returns ['.tar', '.gz'].

        Returns:
            list[str]: List of all suffixes in order.

        Examples:
        ```python
        >>> from pyopath import PurePath
        >>> PurePath('my/library.tar.gar').suffixes
        ['.tar', '.gar']
        >>> PurePath('my/library.tar.gz').suffixes
        ['.tar', '.gz']

        ```
        """
    @property
    def stem(self) -> str:
        """The final path component without its suffix.

        For a path like 'setup.py', this returns 'setup'. For a file like
        'library.tar.gz', this returns 'library.tar'.

        Returns:
            str: The final component without its suffix.

        Examples:
        ```python
        >>> from pyopath import PurePath
        >>> PurePath('my/library.tar.gz').stem
        'library.tar'
        >>> PurePath('my/library').stem
        'library'

        ```
        """

    def as_posix(self) -> str:
        """Return the string representation of the path with forward slashes.

        On Windows, this converts backslashes to forward slashes. On POSIX
        systems, the result is identical to `str(self)`.

        Returns:
            str: The path with forward slashes.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('windows').as_posix()
        'windows'

        ```
        """
    def as_uri(self) -> str:
        """Represent the path as a 'file' URI (RFC 8089).

        The path must be absolute, otherwise `ValueError` is raised.

        Returns:
            str: The path represented as a file URI.

        Raises:
            ValueError: If the path is not absolute.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/home').as_uri()
        'file:///home'

        ```
        """
    def is_absolute(self) -> bool:
        """Check if the path is absolute.

        A path is considered absolute if it has both a root and (if the flavor
        allows) a drive.

        Returns:
            bool: `True` if the path is absolute, `False` otherwise.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/a/b').is_absolute()
        True
        >>> PurePosixPath('a/b').is_absolute()
        False

        ```
        """
    def is_relative_to(self, other: StrPath) -> bool:
        """Check if this path is relative to **other**.

        This method is string-based and doesn't access the filesystem. It
        doesn't treat '..' segments specially.

        Args:
            other (str | PathLike): The reference path.

        Returns:
            bool: `True` if this path is relative to other, `False` otherwise.

        Examples:
        ```python
        >>> from pyopath import PurePath
        >>> PurePath('/etc/passwd').is_relative_to('/etc')
        True
        >>> PurePath('/etc/passwd').is_relative_to('/usr')
        False

        ```
        """
    def match(
        self, path_pattern: str, *, case_sensitive: bool | None = None
    ) -> bool: ...
    def relative_to(self, other: StrPath, *, walk_up: bool = False) -> Self:
        """Compute a version of this path relative to **other**.

        When `walk_up` is `False` (default), the path must start with **other**.
        When `walk_up` is `True`, '..' entries may be added to form the relative
        path.

        Note:
            This is a lexical operation and doesn't check the filesystem.

        Args:
            other (StrPath): The reference path.
            walk_up (bool): If `True`, '..' entries may be added. Defaults to `False`.

        Returns:
            Self: A new path representing the relative path.

        Raises:
            ValueError: If the path cannot be made relative to **other**.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/etc/passwd').relative_to('/')
        PurePosixPath('etc/passwd')
        >>> PurePosixPath('/etc/passwd').relative_to('/etc')
        PurePosixPath('passwd')

        ```
        """
    def with_name(self, name: str) -> Self:
        """Return a new path with the name changed.

        The name is the final path component. If the original path doesn't
        have a name (e.g., it's a root), `ValueError` is raised.

        Args:
            name (str): The new name for the path.

        Returns:
            Self: A new path with the changed name.

        Raises:
            ValueError: If the path has no name (is a root).

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('Downloads/pathlib.tar.gz').with_name('setup.py')
        PurePosixPath('Downloads/setup.py')

        ```
        """
    def with_stem(self, stem: str) -> Self:
        """Return a new path with the stem changed.

        The stem is the final path component without its suffixes. If the
        original path doesn't have a name, `ValueError` is raised.

        Args:
            stem (str): The new stem for the path.

        Returns:
            Self: A new path with the changed stem.

        Raises:
            ValueError: If the path has no name (is a root).

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('Downloads/draft.txt').with_stem('final')
        PurePosixPath('Downloads/final.txt')
        >>> PurePosixPath('Downloads/pathlib.tar.gz').with_stem('lib')
        PurePosixPath('Downloads/lib.gz')

        ```
        """
    def with_suffix(self, suffix: str) -> Self:
        """Return a new path with the suffix changed.

        If the original path doesn't have a suffix, the new suffix is appended
        instead. An empty string removes the suffix.

        Args:
            suffix (str): The new suffix (e.g., '.txt'), or empty string to remove.

        Returns:
            Self: A new path with the changed suffix.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/Downloads/pathlib.tar.gz').with_suffix('.bz2')
        PurePosixPath('/Downloads/pathlib.tar.bz2')
        >>> PurePosixPath('README').with_suffix('.txt')
        PurePosixPath('README.txt')

        ```
        """
    def joinpath(self, *other: StrPath) -> Self:
        """Join path segments to this path.

        This is equivalent to using the `/` operator multiple times.

        Args:
            *other: Path segments to append.

        Returns:
            Self: A new path with the segments combined.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/etc').joinpath('passwd')
        PurePosixPath('/etc/passwd')
        >>> PurePosixPath('/etc').joinpath('init.d', 'apache2')
        PurePosixPath('/etc/init.d/apache2')

        ```
        """
    @property
    def parents(self) -> Sequence[Self]:
        """Immutable sequence providing access to logical ancestors of the path.

        Returns a sequence where index 0 is the immediate parent, index 1 is
        the grandparent, etc. You cannot go past the anchor (root) of the path.

        Returns:
            Sequence[Self]: A sequence of ancestor paths.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> p = PurePosixPath('/foo/bar/setup.py')
        >>> p.parents[0]
        PurePosixPath('/foo/bar')
        >>> p.parents[1]
        PurePosixPath('/foo')
        ```
        """
    @property
    def parent(self) -> Self:
        """The logical parent of the path.

        You cannot go past an anchor (root) or empty path; the parent of a root
        returns the root itself, and the parent of '.' returns '.'.

        Note:
            This is a lexical operation only; '..' components are not resolved.

        Returns:
            Self: The parent path.

        Examples:
        ```python
        >>> from pyopath import PurePosixPath
        >>> PurePosixPath('/a/b/c/d').parent
        PurePosixPath('/a/b/c')
        >>> PurePosixPath('/').parent
        PurePosixPath('/')

        ```
        """
    def with_segments(self, *args: StrPath) -> Self:
        """Create a new path object of the same type by combining segments.

        This method is called internally whenever a derivative path is created.
        Subclasses may override this method to pass information to derivative paths.

        Args:
            *args: Path segments to combine.

        Returns:
            Self: A new path of the same type.
        """

class PurePosixPath(PurePath):
    __slots__ = ()

class PureWindowsPath(PurePath):
    __slots__ = ()

class Path(PurePath):
    __slots__ = ("_info",)
    def __new__(cls, *args: StrPath, **kwargs: Unused) -> Self: ...  # pyright: ignore[reportInconsistentConstructor]
    @classmethod
    def cwd(cls) -> Self: ...
    def stat(self, *, follow_symlinks: bool = True) -> stat_result: ...
    def chmod(self, mode: int, *, follow_symlinks: bool = True) -> None: ...
    @classmethod
    def from_uri(cls, uri: str) -> Self: ...
    def is_dir(self, *, follow_symlinks: bool = True) -> bool: ...
    def is_file(self, *, follow_symlinks: bool = True) -> bool: ...
    def read_text(
        self,
        encoding: str | None = None,
        errors: str | None = None,
        newline: str | None = None,
    ) -> str: ...
    def glob(
        self,
        pattern: str,
        *,
        case_sensitive: bool | None = None,
        recurse_symlinks: bool = False,
    ) -> Iterator[Self]: ...
    def rglob(
        self,
        pattern: str,
        *,
        case_sensitive: bool | None = None,
        recurse_symlinks: bool = False,
    ) -> Iterator[Self]: ...
    def exists(self, *, follow_symlinks: bool = True) -> bool: ...
    def is_symlink(self) -> bool: ...
    def is_socket(self) -> bool: ...
    def is_fifo(self) -> bool: ...
    def is_block_device(self) -> bool: ...
    def is_char_device(self) -> bool: ...
    def is_junction(self) -> bool: ...
    def iterdir(self) -> Generator[Self]: ...
    def lchmod(self, mode: int) -> None: ...
    def lstat(self) -> stat_result: ...
    def mkdir(
        self, mode: int = 0o777, parents: bool = False, exist_ok: bool = False
    ) -> None: ...
    @property
    def info(self) -> PathInfo: ...
    @overload
    def move_into[T: PurePath](self, target_dir: T) -> T: ...  # type: ignore[overload-overlap]
    @overload
    def move_into(self, target_dir: StrPath) -> Self: ...  # type: ignore[overload-overlap]
    @overload
    def move[T: PurePath](self, target: T) -> T: ...  # type: ignore[overload-overlap]
    @overload
    def move(self, target: StrPath) -> Self: ...  # type: ignore[overload-overlap]
    @overload
    def copy_into[T: PurePath](  # type: ignore[overload-overlap]
        self,
        target_dir: T,
        *,
        follow_symlinks: bool = True,
        preserve_metadata: bool = False,
    ) -> T: ...
    @overload
    def copy_into(
        self,
        target_dir: StrPath,
        *,
        follow_symlinks: bool = True,
        preserve_metadata: bool = False,
    ) -> Self: ...  # type: ignore[overload-overlap]
    @overload
    def copy[T: PurePath](  # type: ignore[overload-overlap]
        self,
        target: T,
        *,
        follow_symlinks: bool = True,
        preserve_metadata: bool = False,
    ) -> T: ...
    @overload
    def copy(
        self,
        target: StrPath,
        *,
        follow_symlinks: bool = True,
        preserve_metadata: bool = False,
    ) -> Self: ...  # type: ignore[overload-overlap]

    # Adapted from builtins.open
    # Text mode: always returns a TextIOWrapper
    # The Traversable .open in stdlib/importlib/abc.pyi should be kept in sync with this.
    @overload
    def open(
        self,
        mode: OpenTextMode = "r",
        buffering: int = -1,
        encoding: str | None = None,
        errors: str | None = None,
        newline: str | None = None,
    ) -> TextIOWrapper: ...
    # Unbuffered binary mode: returns a FileIO
    @overload
    def open(
        self,
        mode: OpenBinaryMode,
        buffering: Literal[0],
        encoding: None = None,
        errors: None = None,
        newline: None = None,
    ) -> FileIO: ...
    # Buffering is on: return BufferedRandom, BufferedReader, or BufferedWriter
    @overload
    def open(
        self,
        mode: OpenBinaryModeUpdating,
        buffering: Literal[-1, 1] = -1,
        encoding: None = None,
        errors: None = None,
        newline: None = None,
    ) -> BufferedRandom: ...
    @overload
    def open(
        self,
        mode: OpenBinaryModeWriting,
        buffering: Literal[-1, 1] = -1,
        encoding: None = None,
        errors: None = None,
        newline: None = None,
    ) -> BufferedWriter: ...
    @overload
    def open(
        self,
        mode: OpenBinaryModeReading,
        buffering: Literal[-1, 1] = -1,
        encoding: None = None,
        errors: None = None,
        newline: None = None,
    ) -> BufferedReader: ...
    # Buffering cannot be determined: fall back to BinaryIO
    @overload
    def open(
        self,
        mode: OpenBinaryMode,
        buffering: int = -1,
        encoding: None = None,
        errors: None = None,
        newline: None = None,
    ) -> BinaryIO: ...
    # Fallback if mode is not specified
    @overload
    def open(
        self,
        mode: str,
        buffering: int = -1,
        encoding: str | None = None,
        errors: str | None = None,
        newline: str | None = None,
    ) -> IO[Any]: ...

    # These methods do "exist" on Windows, but they always raise NotImplementedError.
    if sys.platform == "win32":
        # raises UnsupportedOperation:
        def owner(self: Never, *, follow_symlinks: bool = True) -> str: ...  # type: ignore[misc]
        def group(self: Never, *, follow_symlinks: bool = True) -> str: ...  # type: ignore[misc]
    def owner(self, *, follow_symlinks: bool = True) -> str: ...
    def group(self, *, follow_symlinks: bool = True) -> str: ...
    def is_mount(self) -> bool: ...
    def readlink(self) -> Self: ...
    def rename(self, target: StrPath) -> Self: ...
    def replace(self, target: StrPath) -> Self: ...
    def resolve(self, strict: bool = False) -> Self: ...
    def rmdir(self) -> None: ...
    def symlink_to(
        self, target: StrOrBytesPath, target_is_directory: bool = False
    ) -> None: ...
    def hardlink_to(self, target: StrOrBytesPath) -> None: ...
    def touch(self, mode: int = 0o666, exist_ok: bool = True) -> None: ...
    def unlink(self, missing_ok: bool = False) -> None: ...
    @classmethod
    def home(cls) -> Self: ...
    def absolute(self) -> Self: ...
    def expanduser(self) -> Self: ...
    def read_bytes(self) -> bytes: ...
    def samefile(self, other_path: StrPath) -> bool: ...
    def write_bytes(self, data: ReadableBuffer) -> int: ...
    def write_text(
        self,
        data: str,
        encoding: str | None = None,
        errors: str | None = None,
        newline: str | None = None,
    ) -> int: ...
    def walk(
        self,
        top_down: bool = True,
        on_error: Callable[[OSError], object] | None = None,
        follow_symlinks: bool = False,
    ) -> Iterator[tuple[Self, list[str], list[str]]]: ...

class PosixPath(Path, PurePosixPath):
    __slots__ = ()

class WindowsPath(Path, PurePosixPath):
    __slots__ = ()

class UnsupportedOperation(NotImplementedError): ...  # noqa: N818
