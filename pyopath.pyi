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
    "PureWindowsPath",
    "WindowsPath",
]

class PurePath(PathLike[str]):
    __slots__ = (
        "_drv",
        "_hash",
        "_parts_normcase_cached",
        "_raw_paths",
        "_root",
        "_str",
        "_str_normcase_cached",
        "_tail_cached",
    )
    parser: ClassVar[types.ModuleType]
    def full_match(
        self, pattern: StrPath, *, case_sensitive: bool | None = None
    ) -> bool: ...
    @property
    def parts(self) -> tuple[str, ...]: ...
    @property
    def drive(self) -> str: ...
    @property
    def root(self) -> str: ...
    @property
    def anchor(self) -> str: ...
    @property
    def name(self) -> str: ...
    @property
    def suffix(self) -> str: ...
    @property
    def suffixes(self) -> list[str]: ...
    @property
    def stem(self) -> str: ...
    def __new__(cls, *args: StrPath, **kwargs: Unused) -> Self: ...
    def __init__(self, *args: StrPath) -> None: ...  # pyright: ignore[reportInconsistentConstructor]
    def __hash__(self) -> int: ...
    def __fspath__(self) -> str: ...
    def __lt__(self, other: PurePath) -> bool: ...
    def __le__(self, other: PurePath) -> bool: ...
    def __gt__(self, other: PurePath) -> bool: ...
    def __ge__(self, other: PurePath) -> bool: ...
    def __truediv__(self, key: StrPath) -> Self: ...
    def __rtruediv__(self, key: StrPath) -> Self: ...
    def __bytes__(self) -> bytes: ...
    def as_posix(self) -> str: ...
    def as_uri(self) -> str: ...
    def is_absolute(self) -> bool: ...
    def is_relative_to(self, other: StrPath) -> bool: ...
    def match(
        self, path_pattern: str, *, case_sensitive: bool | None = None
    ) -> bool: ...
    def relative_to(self, other: StrPath, *, walk_up: bool = False) -> Self: ...
    def with_name(self, name: str) -> Self: ...
    def with_stem(self, stem: str) -> Self: ...
    def with_suffix(self, suffix: str) -> Self: ...
    def joinpath(self, *other: StrPath) -> Self: ...
    @property
    def parents(self) -> Sequence[Self]: ...
    @property
    def parent(self) -> Self: ...
    def with_segments(self, *args: StrPath) -> Self: ...

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

class WindowsPath(Path, PureWindowsPath):
    __slots__ = ()

class UnsupportedOperation(NotImplementedError): ...  # noqa: N818
