# type: ignore[misc]
"""This module exists so that pathlib objects pickled under Python 3.13 can be
unpickled in 3.14+.
"""

from pathlib import *

__all__ = [
    "Path",
    "PosixPath",
    "PurePath",
    "PurePosixPath",
    "PureWindowsPath",
    "UnsupportedOperation",
    "WindowsPath",
]
