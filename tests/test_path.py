"""Tests for Path and its subclasses (filesystem operations)."""

import pathlib
import tempfile
from collections.abc import Generator

import pyochain as pc
import pyopath
import pytest


class TestPathFilesystemQueries:
    """Test Path filesystem query methods."""

    @pytest.fixture
    def temp_dir(self) -> Generator[pc.Iter[pyopath.Path]]:
        """Create a temporary directory with test files."""
        with tempfile.TemporaryDirectory() as tmp:
            base = pyopath.Path(tmp)
            # Create test structure
            (base / "file.txt").write_text("hello")
            (base / "subdir").mkdir()
            (base / "subdir" / "nested.py").write_text("# python")
            yield pc.Iter.once(base)

    def test_exists(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test exists method."""
        base = temp_dir.first()
        assert (base / "file.txt").exists()
        assert not (base / "nonexistent").exists()

    def test_is_file(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test is_file method."""
        base = temp_dir.first()
        assert (base / "file.txt").is_file()
        assert not (base / "subdir").is_file()

    def test_is_dir(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test is_dir method."""
        base = temp_dir.first()
        assert (base / "subdir").is_dir()
        assert not (base / "file.txt").is_dir()

    def test_stat(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test stat method."""
        base = temp_dir.first()
        file_path = base / "file.txt"
        stat_result = file_path.stat()

        hello_bytes = 5

        assert stat_result.st_size == hello_bytes  # "hello" is 5 bytes
        assert stat_result.st_mtime > 0
        assert stat_result.st_atime > 0


class TestPathResolution:
    """Test Path resolution methods."""

    def test_absolute(self) -> None:
        """Test absolute method."""
        rel_path = pyopath.Path("relative/path")
        abs_path = rel_path.absolute()
        assert abs_path.is_absolute()

    def test_resolve(self) -> None:
        """Test resolve method."""
        # resolve should return absolute path without symlinks
        p = pyopath.Path(".")
        resolved = p.resolve()
        assert resolved.is_absolute()

    def test_cwd(self) -> None:
        """Test cwd static method."""
        cwd = pyopath.Path.cwd()
        assert cwd.is_absolute()
        assert cwd.exists()
        assert str(cwd) == str(pathlib.Path.cwd())

    def test_home(self) -> None:
        """Test home static method."""
        home = pyopath.Path.home()
        assert home.is_absolute()
        assert home.exists()
        assert str(home) == str(pathlib.Path.home())


class TestPathDirectoryOps:
    """Test Path directory operations."""

    @pytest.fixture
    def temp_dir(self) -> Generator[pc.Iter[pyopath.Path]]:
        """Create a temporary directory."""
        with tempfile.TemporaryDirectory() as tmp:
            yield pc.Iter.once(pyopath.Path(tmp))

    def test_mkdir(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test mkdir method."""
        base = temp_dir.first()
        new_dir = base / "new_directory"
        new_dir.mkdir()
        assert new_dir.exists()
        assert new_dir.is_dir()

    def test_mkdir_parents(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test mkdir with parents=True."""
        base = temp_dir.first()
        nested = base / "a" / "b" / "c"
        nested.mkdir(parents=True)
        assert nested.exists()

    def test_mkdir_exist_ok(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test mkdir with exist_ok=True."""
        base = temp_dir.first()
        existing = base / "existing"
        existing.mkdir()
        # Should not raise
        existing.mkdir(exist_ok=True)

    def test_rmdir(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test rmdir method."""
        base = temp_dir.first()
        to_remove = base / "to_remove"
        to_remove.mkdir()
        assert to_remove.exists()
        to_remove.rmdir()
        assert not to_remove.exists()

    def test_iterdir(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test iterdir method."""
        base = temp_dir.first()
        # Create some files
        (base / "file1.txt").touch()
        (base / "file2.txt").touch()
        (base / "subdir").mkdir()

        entries = pc.Iter(base.iterdir()).map(lambda p: p.name).collect(pc.Set)
        assert "file1.txt" in entries
        assert "file2.txt" in entries
        assert "subdir" in entries


class TestPathGlobbing:
    """Test Path globbing methods."""

    @pytest.fixture
    def temp_dir(self) -> Generator[pc.Iter[pyopath.Path]]:
        """Create a temporary directory with files for globbing."""
        with tempfile.TemporaryDirectory() as tmp:
            base = pyopath.Path(tmp)
            # Create test structure
            (base / "file1.txt").touch()
            (base / "file2.txt").touch()
            (base / "data.json").touch()
            (base / "subdir").mkdir()
            (base / "subdir" / "nested.txt").touch()
            (base / "subdir" / "deep").mkdir()
            (base / "subdir" / "deep" / "file.txt").touch()
            yield pc.Iter.once(base)

    def test_glob(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test glob method."""
        base = temp_dir.first()
        txt_files = pc.Iter(base.glob("*.txt")).map(lambda p: p.name).collect(pc.Set)
        assert "file1.txt" in txt_files
        assert "file2.txt" in txt_files
        assert "data.json" not in txt_files

    def test_rglob(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test rglob method (recursive glob)."""
        base = temp_dir.first()
        all_txt = pc.Iter(base.rglob("*.txt")).map(lambda p: p.name).collect(pc.Set)
        assert "file1.txt" in all_txt
        assert "nested.txt" in all_txt
        assert "file.txt" in all_txt  # deep nested


class TestPathFileOps:
    """Test Path file operations."""

    @pytest.fixture
    def temp_dir(self) -> Generator[pc.Iter[pyopath.Path]]:
        """Create a temporary directory."""
        with tempfile.TemporaryDirectory() as tmp:
            yield pc.Iter.once(pyopath.Path(tmp))

    def test_touch(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test touch method."""
        base = temp_dir.first()
        new_file = base / "touched.txt"
        assert not new_file.exists()
        new_file.touch()
        assert new_file.exists()

    def test_unlink(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test unlink method."""
        base = temp_dir.first()
        file_to_delete = base / "to_delete.txt"
        file_to_delete.touch()
        assert file_to_delete.exists()
        file_to_delete.unlink()
        assert not file_to_delete.exists()

    def test_unlink_missing_ok(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test unlink with missing_ok=True."""
        base = temp_dir.first()
        nonexistent = base / "nonexistent.txt"
        # Should not raise
        nonexistent.unlink(missing_ok=True)

    def test_rename(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test rename method."""
        base = temp_dir.first()
        original = base / "original.txt"
        original.write_text("content")
        renamed = base / "renamed.txt"

        result = original.rename(renamed)
        assert not original.exists()
        assert renamed.exists()
        assert str(result) == str(renamed)

    def test_replace(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test replace method."""
        base = temp_dir.first()
        src = base / "source.txt"
        dst = base / "destination.txt"
        src.write_text("source content")
        dst.write_text("dest content")

        src.replace(dst)
        assert not src.exists()
        assert dst.read_text() == "source content"


class TestPathReadWrite:
    """Test Path read/write operations."""

    @pytest.fixture
    def temp_dir(self) -> Generator[pc.Iter[pyopath.Path]]:
        """Create a temporary directory."""
        with tempfile.TemporaryDirectory() as tmp:
            yield pc.Iter.once(pyopath.Path(tmp))

    def test_read_write_text(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test read_text and write_text methods."""
        base = temp_dir.first()
        file_path = base / "text.txt"
        content = "Hello, World!\nLine 2"

        bytes_written = file_path.write_text(content)
        assert bytes_written == len(content)

        read_content = file_path.read_text()
        assert read_content == content

    def test_read_write_bytes(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test read_bytes and write_bytes methods."""
        base = temp_dir.first()
        file_path = base / "binary.bin"
        content = b"\x00\x01\x02\xff\xfe"

        bytes_written = file_path.write_bytes(content)
        assert bytes_written == len(content)

        read_content = file_path.read_bytes()
        assert read_content == content

    def test_open(self, temp_dir: pc.Iter[pyopath.Path]) -> None:
        """Test open method."""
        base = temp_dir.first()
        file_path = base / "opened.txt"

        with file_path.open("w") as f:
            f.write("written via open")

        with file_path.open("r") as f:
            content = f.read()

        assert content == "written via open"


class TestPathCompatibility:
    """Test compatibility between pyopath and pathlib."""

    @pytest.fixture
    def temp_dir(self) -> Generator[pc.Iter[tuple[pyopath.Path, pathlib.Path]]]:
        """Create matching pyopath and pathlib paths."""
        with tempfile.TemporaryDirectory() as tmp:
            yield pc.Iter.once((pyopath.Path(tmp), pathlib.Path(tmp)))

    def test_same_behavior(
        self, temp_dir: pc.Iter[tuple[pyopath.Path, pathlib.Path]]
    ) -> None:
        """Test that pyopath and pathlib behave identically."""
        pyopath_base, pathlib_base = temp_dir.first()

        # Create same structure
        (pyopath_base / "test.txt").write_text("hello")
        (pathlib_base / "test2.txt").write_text("world")

        # Both should see all files
        pyopath_files = pc.Iter(pyopath_base.iterdir()).map(lambda p: p.name).sort()
        pathlib_files = pc.Iter(pathlib_base.iterdir()).map(lambda p: p.name).sort()

        assert pyopath_files.eq(pathlib_files)
