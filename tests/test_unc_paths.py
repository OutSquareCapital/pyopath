"""Tests for UNC paths and Windows path handling - comparing with pathlib."""

from pathlib import PureWindowsPath as StdPureWindowsPath

import pyopath


class TestUNCPaths:
    """Test UNC path handling against pathlib."""

    def test_unc_basic(self) -> None:
        """Test basic UNC path."""
        p_pyopath = pyopath.PureWindowsPath(r"\\server\share\file.txt")
        p_pathlib = StdPureWindowsPath(r"\\server\share\file.txt")

        assert p_pyopath.drive == p_pathlib.drive
        assert p_pyopath.root == p_pathlib.root
        assert p_pyopath.name == p_pathlib.name

    def test_unc_with_forward_slashes(self) -> None:
        """Test UNC path with forward slashes."""
        p_pyopath = pyopath.PureWindowsPath("//server/share/file.txt")
        p_pathlib = StdPureWindowsPath("//server/share/file.txt")

        assert p_pyopath.drive == p_pathlib.drive
        assert p_pyopath.root == p_pathlib.root
        assert p_pyopath.name == p_pathlib.name

    def test_unc_directory(self) -> None:
        """Test UNC path to directory."""
        p_pyopath = pyopath.PureWindowsPath(r"\\server\share\folder")
        p_pathlib = StdPureWindowsPath(r"\\server\share\folder")

        assert p_pyopath.drive == p_pathlib.drive
        assert p_pyopath.root == p_pathlib.root
        assert p_pyopath.name == p_pathlib.name

    def test_unc_parts(self) -> None:
        """Test UNC path parts."""
        p_pyopath = pyopath.PureWindowsPath(r"\\server\share\folder\file.txt")
        p_pathlib = StdPureWindowsPath(r"\\server\share\folder\file.txt")

        assert list(p_pyopath.parts) == list(p_pathlib.parts)

    def test_drive_with_backslash(self) -> None:
        """Test regular drive letter path."""
        p_pyopath = pyopath.PureWindowsPath(r"C:\Windows\System32")
        p_pathlib = StdPureWindowsPath(r"C:\Windows\System32")

        assert p_pyopath.drive == p_pathlib.drive
        assert p_pyopath.root == p_pathlib.root
        assert p_pyopath.name == p_pathlib.name

    def test_drive_with_forward_slash(self) -> None:
        """Test drive letter path with forward slashes."""
        p_pyopath = pyopath.PureWindowsPath("C:/Windows/System32")
        p_pathlib = StdPureWindowsPath("C:/Windows/System32")

        assert p_pyopath.drive == p_pathlib.drive
        assert p_pyopath.root == p_pathlib.root
        assert p_pyopath.name == p_pathlib.name


class TestWindowsPathConstruction:
    """Test Windows path construction with multiple args."""

    def test_absolute_resets_previous(self) -> None:
        """Test that absolute path resets previous path."""
        p_pyopath = pyopath.PureWindowsPath("c:/Windows", "d:bar")
        p_pathlib = StdPureWindowsPath("c:/Windows", "d:bar")

        assert str(p_pyopath) == str(p_pathlib)

    def test_unc_resets_previous(self) -> None:
        """Test that UNC path resets previous path."""
        p_pyopath = pyopath.PureWindowsPath("c:/Windows", r"\\server\share")
        p_pathlib = StdPureWindowsPath("c:/Windows", r"\\server\share")

        assert p_pyopath.drive == p_pathlib.drive
        assert p_pyopath.root == p_pathlib.root
