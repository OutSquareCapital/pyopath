"""Tests for PurePath and its subclasses."""

import pathlib

import pyochain as pc
import pyopath
import pytest


class TestPurePathProperties:
    """Test PurePath property behavior matches pathlib."""

    @pytest.fixture
    def test_paths(self) -> pc.Seq[str]:
        """Sample paths for testing."""
        return pc.Seq(
            (
                "/usr/local/bin",
                "C:/Users/test",
                "relative/path/file.txt",
                "/home/user/.config",
                "file.tar.gz",
                ".",
                "..",
                "/",
            )
        )

    def test_parts(self, test_paths: pc.Seq[str]) -> None:
        """Test parts property."""

        def _check(p: str) -> None:
            assert tuple(pyopath.PurePath(p).parts) == pathlib.PurePath(p).parts

        test_paths.iter().for_each(_check)

    def test_name(self, test_paths: pc.Seq[str]) -> None:
        """Test name property."""

        def _check(p: str) -> None:
            assert pyopath.PurePath(p).name == pathlib.PurePath(p).name

        test_paths.iter().for_each(_check)

    def test_suffix(self, test_paths: pc.Seq[str]) -> None:
        """Test suffix property."""

        def _check(p: str) -> None:
            assert pyopath.PurePath(p).suffix == pathlib.PurePath(p).suffix

        test_paths.iter().for_each(_check)

    def test_suffixes(self, test_paths: pc.Seq[str]) -> None:
        """Test suffixes property."""

        def _check(p: str) -> None:
            assert list(pyopath.PurePath(p).suffixes) == pathlib.PurePath(p).suffixes

        test_paths.iter().for_each(_check)

    def test_stem(self, test_paths: pc.Seq[str]) -> None:
        """Test stem property."""

        def _check(p: str) -> None:
            assert pyopath.PurePath(p).stem == pathlib.PurePath(p).stem

        test_paths.iter().for_each(_check)

    def test_parent(self, test_paths: pc.Seq[str]) -> None:
        """Test parent property."""

        def _check(p: str) -> None:
            assert str(pyopath.PurePath(p).parent) == str(pathlib.PurePath(p).parent)

        test_paths.iter().for_each(_check)

    def test_parents(self) -> None:
        """Test parents property."""
        path = "/usr/local/bin/python"
        pyopath_parents = pc.Iter(pyopath.PurePath(path).parents).map(str).collect()
        pathlib_parents = pc.Iter(pathlib.PurePath(path).parents).map(str).collect()
        assert pyopath_parents.eq(pathlib_parents)

    def test_anchor(self, test_paths: pc.Seq[str]) -> None:
        """Test anchor property."""

        def _check(p: str) -> None:
            assert pyopath.PurePath(p).anchor == pathlib.PurePath(p).anchor

        test_paths.iter().for_each(_check)

    def test_drive(self) -> None:
        """Test drive property (Windows paths)."""
        windows_paths = pc.Seq(("C:/Users", "D:/Data", "/unix/path", "relative"))

        def _check(p: str) -> None:
            assert pyopath.PureWindowsPath(p).drive == pathlib.PureWindowsPath(p).drive

        windows_paths.iter().for_each(_check)

    def test_root(self, test_paths: pc.Seq[str]) -> None:
        """Test root property."""

        def _check(p: str) -> None:
            assert pyopath.PurePath(p).root == pathlib.PurePath(p).root

        test_paths.iter().for_each(_check)


class TestPurePathMethods:
    """Test PurePath method behavior matches pathlib."""

    def test_is_absolute(self) -> None:
        """Test is_absolute method."""
        paths = pc.Seq(("/absolute/path", "relative/path", "C:/windows", "."))

        def _check(p: str) -> None:
            assert (
                pyopath.PurePath(p).is_absolute() == pathlib.PurePath(p).is_absolute()
            )

        paths.iter().for_each(_check)

    def test_is_relative_to(self) -> None:
        """Test is_relative_to method."""
        assert pyopath.PurePath("/usr/local/bin").is_relative_to("/usr")
        assert not pyopath.PurePath("/usr/local/bin").is_relative_to("/etc")
        assert pyopath.PurePath("foo/bar").is_relative_to("foo")

    def test_relative_to(self) -> None:
        """Test relative_to method."""
        assert str(pyopath.PurePath("/usr/local/bin").relative_to("/usr")) == str(
            pathlib.PurePath("/usr/local/bin").relative_to("/usr")
        )

    def test_joinpath(self) -> None:
        """Test joinpath method."""
        base = "/home/user"
        parts = pc.Seq(("documents", "file.txt"))
        pyopath_result = str(pyopath.PurePath(base).joinpath(*parts))
        pathlib_result = str(pathlib.PurePath(base).joinpath(*parts))
        assert pyopath_result == pathlib_result

    def test_truediv_operator(self) -> None:
        """Test / operator."""
        pyopath_result = str(pyopath.PurePath("/home") / "user" / "file.txt")
        pathlib_result = str(pathlib.PurePath("/home") / "user" / "file.txt")
        assert pyopath_result == pathlib_result

    def test_with_name(self) -> None:
        """Test with_name method."""
        paths = pc.Seq(("/home/file.txt", "dir/doc.pdf", "test.py"))

        def _check(p: str) -> None:
            assert str(pyopath.PurePath(p).with_name("new.ext")) == str(
                pathlib.PurePath(p).with_name("new.ext")
            )

        paths.iter().for_each(_check)

    def test_with_stem(self) -> None:
        """Test with_stem method."""
        paths = pc.Seq(("/home/file.txt", "dir/doc.pdf", "test.py"))

        def _check(p: str) -> None:
            assert str(pyopath.PurePath(p).with_stem("newname")) == str(
                pathlib.PurePath(p).with_stem("newname")
            )

        paths.iter().for_each(_check)

    def test_with_suffix(self) -> None:
        """Test with_suffix method."""
        paths = pc.Seq(("/home/file.txt", "dir/doc.pdf", "test"))

        def _check(p: str) -> None:
            assert str(pyopath.PurePath(p).with_suffix(".new")) == str(
                pathlib.PurePath(p).with_suffix(".new")
            )

        paths.iter().for_each(_check)

    def test_as_posix(self) -> None:
        """Test as_posix method."""
        assert pyopath.PureWindowsPath("C:\\Users\\test").as_posix() == "C:/Users/test"
        assert pyopath.PurePath("/unix/path").as_posix() == "/unix/path"


class TestPurePathComparison:
    """Test PurePath comparison operations."""

    def test_equality(self) -> None:
        """Test equality comparison."""
        assert pyopath.PurePath("/home/user") == pyopath.PurePath("/home/user")
        assert pyopath.PurePath("/home/user") != pyopath.PurePath("/home/other")

    def test_inequality(self) -> None:
        """Test inequality comparison."""
        assert pyopath.PurePath("/home/user") != pyopath.PurePath("/home/other")

    def test_ordering(self) -> None:
        """Test ordering comparisons."""
        paths = pc.Seq(
            (
                pyopath.PurePath("/a"),
                pyopath.PurePath("/b"),
                pyopath.PurePath("/c"),
            )
        )
        assert paths.nth(0) < paths.nth(1)
        assert paths.nth(2) > paths.nth(1)
        assert paths.nth(0) <= paths.nth(0)
        assert paths.nth(2) >= paths.nth(1)

    def test_hash(self) -> None:
        """Test hash consistency."""
        p1 = pyopath.PurePath("/home/user")
        p2 = pyopath.PurePath("/home/user")
        assert hash(p1) == hash(p2)

        # Can be used in sets
        path_set = {p1, p2}
        assert len(path_set) == 1


class TestPurePathFspath:
    """Test os.fspath compatibility."""

    def test_fspath(self) -> None:
        """Test __fspath__ method."""
        import os

        # On Windows, PurePath uses backslashes (native separator)
        # So we compare with pathlib's behavior which is platform-dependent
        p = pyopath.PurePath("/home/user/file.txt")
        expected = str(pathlib.PurePath("/home/user/file.txt"))
        assert os.fspath(p) == expected
