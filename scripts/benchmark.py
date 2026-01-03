"""Benchmark pyopath vs pathlib performance."""

import pathlib
import tempfile
import timeit
from collections.abc import Callable
from dataclasses import dataclass
from enum import StrEnum, auto
from pathlib import Path

import polars as pl
import pyochain as pc
import pyopath

DATA = Path("scripts", "data")
ITERATIONS = 1000


class Category(StrEnum):
    """Benchmark category."""

    PURE_PATH = auto()
    FILESYSTEM = auto()


DF_SCHEMA = {
    "name": pl.String,
    "category": pl.Enum(Category),
    "iterations": pl.Int64,
    "speedup": pl.Float64,
    "faster": pl.String,
}


@dataclass(slots=True)
class BenchmarkResult:
    """Result of a benchmark comparison."""

    name: str
    category: Category
    pyopath_us: float
    pathlib_us: float
    iterations: int = ITERATIONS

    def to_row(self) -> tuple[str, Category, int, float, str]:
        """Convert to row tuple for polars."""
        speedup = self.pathlib_us / self.pyopath_us
        faster = "pyopath" if speedup > 1 else "pathlib"
        return (
            self.name,
            self.category,
            self.iterations,
            speedup,
            faster,
        )


def _compare(
    name: str,
    category: Category,
    pyopath_func: Callable[[], object],
    pathlib_func: Callable[[], object],
) -> BenchmarkResult:
    """Compare pyopath and pathlib for a given operation."""
    pyopath_s = timeit.timeit(pyopath_func, number=ITERATIONS)
    pathlib_s = timeit.timeit(pathlib_func, number=ITERATIONS)

    return BenchmarkResult(
        name=name,
        category=category,
        iterations=ITERATIONS,
        pyopath_us=(pyopath_s * 1_000_000) / ITERATIONS,
        pathlib_us=(pathlib_s * 1_000_000) / ITERATIONS,
    )


def benchmark_pure_path_creation() -> BenchmarkResult:
    """Benchmark path creation."""
    return _compare(
        "PurePath creation",
        Category.PURE_PATH,
        lambda: pyopath.PurePath("/home/user/documents/file.txt"),
        lambda: pathlib.PurePath("/home/user/documents/file.txt"),
    )


def benchmark_path_parts() -> BenchmarkResult:
    """Benchmark accessing path parts."""
    pyopath_p = pyopath.PurePath("/home/user/documents/project/src/main.py")
    pathlib_p = pathlib.PurePath("/home/user/documents/project/src/main.py")

    return _compare(
        "parts property",
        Category.PURE_PATH,
        lambda: pyopath_p.parts,
        lambda: pathlib_p.parts,
    )


def benchmark_joinpath() -> BenchmarkResult:
    """Benchmark joinpath operation."""
    pyopath_base = pyopath.PurePath("/home/user")
    pathlib_base = pathlib.PurePath("/home/user")

    return _compare(
        "joinpath (str)",
        Category.PURE_PATH,
        lambda: pyopath_base.joinpath("documents", "file.txt"),
        lambda: pathlib_base.joinpath("documents", "file.txt"),
    )


def benchmark_joinpath_path() -> BenchmarkResult:
    """Benchmark joinpath with Path objects (no re-parsing needed)."""
    pyopath_base = pyopath.PurePath("/home/user")
    pathlib_base = pathlib.PurePath("/home/user")
    pyopath_part = pyopath.PurePath("documents/file.txt")
    pathlib_part = pathlib.PurePath("documents/file.txt")

    return _compare(
        "joinpath (Path)",
        Category.PURE_PATH,
        lambda: pyopath_base.joinpath(pyopath_part),
        lambda: pathlib_base.joinpath(pathlib_part),
    )


def benchmark_parent_chain() -> BenchmarkResult:
    """Benchmark traversing parent chain."""
    pyopath_p = pyopath.PurePath("/a/b/c/d/e/f/g/h/i/j")
    pathlib_p = pathlib.PurePath("/a/b/c/d/e/f/g/h/i/j")

    return _compare(
        "parents traversal",
        Category.PURE_PATH,
        lambda: list(pyopath_p.parents),
        lambda: list(pathlib_p.parents),
    )


def benchmark_with_suffix() -> BenchmarkResult:
    """Benchmark with_suffix operation."""
    pyopath_p = pyopath.PurePath("/home/user/file.txt")
    pathlib_p = pathlib.PurePath("/home/user/file.txt")

    return _compare(
        "with_suffix",
        Category.PURE_PATH,
        lambda: pyopath_p.with_suffix(".md"),
        lambda: pathlib_p.with_suffix(".md"),
    )


def benchmark_is_absolute() -> BenchmarkResult:
    """Benchmark is_absolute check."""
    pyopath_p = pyopath.PurePath("/home/user/file.txt")
    pathlib_p = pathlib.PurePath("/home/user/file.txt")

    return _compare(
        "is_absolute",
        Category.PURE_PATH,
        lambda: pyopath_p.is_absolute(),
        lambda: pathlib_p.is_absolute(),
    )


def benchmark_file_read(tmp_path: str) -> BenchmarkResult:
    """Benchmark file reading."""
    pyopath_p = pyopath.Path(tmp_path).joinpath("bench_read.txt")
    pathlib_p = pathlib.Path(tmp_path).joinpath("bench_read.txt")
    pathlib_p.write_text("x" * 1000)

    return _compare(
        "read_text (1KB)",
        Category.FILESYSTEM,
        lambda: pyopath_p.read_text(),
        lambda: pathlib_p.read_text(),
    )


def benchmark_file_write(tmp_path: str) -> BenchmarkResult:
    """Benchmark file writing."""
    pyopath_p = pyopath.Path(tmp_path).joinpath("bench_write_pyopath.txt")
    pathlib_p = pathlib.Path(tmp_path).joinpath("bench_write_pathlib.txt")
    content = "x" * 1000

    return _compare(
        "write_text (1KB)",
        Category.FILESYSTEM,
        lambda: pyopath_p.write_text(content),
        lambda: pathlib_p.write_text(content),
    )


def benchmark_exists(tmp_path: str) -> BenchmarkResult:
    """Benchmark exists check."""
    pyopath_p = pyopath.Path(tmp_path)
    pathlib_p = pathlib.Path(tmp_path)

    return _compare(
        "exists",
        Category.FILESYSTEM,
        lambda: pyopath_p.exists(),
        lambda: pathlib_p.exists(),
    )


def benchmark_glob(tmp_path: str) -> BenchmarkResult:
    """Benchmark glob operation."""
    pyopath_p = pyopath.Path(tmp_path)
    pathlib_p = pathlib.Path(tmp_path)

    # Create some files
    pc.Iter.from_count().take(20).for_each(
        lambda i: pathlib.Path(tmp_path).joinpath(f"file_{i}.txt").touch()
    )

    return _compare(
        "glob *.txt",
        Category.FILESYSTEM,
        lambda: list(pyopath_p.glob("*.txt")),
        lambda: list(pathlib_p.glob("*.txt")),
    )


def benchmark_iterdir(tmp_path: str) -> BenchmarkResult:
    """Benchmark iterdir operation."""
    pyopath_p = pyopath.Path(tmp_path)
    pathlib_p = pathlib.Path(tmp_path)

    return _compare(
        "iterdir",
        Category.FILESYSTEM,
        lambda: list(pyopath_p.iterdir()),
        lambda: list(pathlib_p.iterdir()),
    )


def _collect_all_benchmarks() -> pc.Vec[BenchmarkResult]:
    """Run all benchmarks and collect results."""
    results = pc.Vec[BenchmarkResult].new()

    # Pure path benchmarks
    pc.Iter(
        (
            benchmark_pure_path_creation(),
            benchmark_path_parts(),
            benchmark_joinpath(),
            benchmark_joinpath_path(),
            benchmark_parent_chain(),
            benchmark_with_suffix(),
            benchmark_is_absolute(),
        )
    ).for_each(results.append)

    # Filesystem benchmarks
    with tempfile.TemporaryDirectory() as tmp:
        pc.Iter(
            (
                benchmark_exists(tmp),
                benchmark_file_read(tmp),
                benchmark_file_write(tmp),
                benchmark_glob(tmp),
                benchmark_iterdir(tmp),
            )
        ).for_each(results.append)

    return results


def main() -> None:
    """Run all benchmarks and save to ndjson."""
    DATA.mkdir(parents=True, exist_ok=True)

    _collect_all_benchmarks().iter().map(lambda br: br.to_row()).into(
        lambda results: pl.LazyFrame(
            data=results,
            schema=DF_SCHEMA,
            orient="row",
        )
        .with_columns(pl.col("speedup").round(2))
        .sort("speedup", descending=True)
        .sink_ndjson(DATA.joinpath("benchmark_results.ndjson"))
    )

    print(f"Results saved to {DATA.joinpath('benchmark_results.ndjson')}")


if __name__ == "__main__":
    main()
