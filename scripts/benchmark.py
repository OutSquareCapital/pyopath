"""Benchmark pyopath vs pathlib performance."""

import pathlib
import tempfile
import timeit
from collections.abc import Callable
from dataclasses import dataclass
from typing import Literal

import pyochain as pc
import pyopath

Library = Literal["pyopath", "pathlib"]


@dataclass(slots=True)
class BenchmarkResult:
    """Result of a benchmark run."""

    name: str
    library: Library
    iterations: int
    total_time_ms: float
    avg_time_us: float

    def __str__(self) -> str:
        """Format result for display."""
        return f"{self.library:8} | {self.avg_time_us:10.2f} µs/op | {self.total_time_ms:8.2f} ms total"


def _benchmark(
    name: str,
    func: Callable[[], object],
    library: Library,
    iterations: int = 10000,
) -> BenchmarkResult:
    """Run a benchmark and return results."""
    total_s = timeit.timeit(func, number=iterations)
    total_ms = total_s * 1000
    avg_us = (total_ms * 1000) / iterations

    return BenchmarkResult(
        name=name,
        library=library,
        iterations=iterations,
        total_time_ms=total_ms,
        avg_time_us=avg_us,
    )


def _compare(
    name: str,
    pyopath_func: Callable[[], object],
    pathlib_func: Callable[[], object],
    iterations: int = 10000,
) -> pc.Seq[BenchmarkResult]:
    """Compare pyopath and pathlib for a given operation."""
    return pc.Seq(
        (
            _benchmark(name, pyopath_func, "pyopath", iterations),
            _benchmark(name, pathlib_func, "pathlib", iterations),
        )
    )


def benchmark_pure_path_creation() -> pc.Seq[BenchmarkResult]:
    """Benchmark path creation."""
    return _compare(
        "PurePath creation",
        lambda: pyopath.PurePath("/home/user/documents/file.txt"),
        lambda: pathlib.PurePath("/home/user/documents/file.txt"),
        iterations=100000,
    )


def benchmark_path_parts() -> pc.Seq[BenchmarkResult]:
    """Benchmark accessing path parts."""
    pyopath_p = pyopath.PurePath("/home/user/documents/project/src/main.py")
    pathlib_p = pathlib.PurePath("/home/user/documents/project/src/main.py")

    return _compare(
        "parts property",
        lambda: pyopath_p.parts,
        lambda: pathlib_p.parts,
        iterations=100000,
    )


def benchmark_joinpath() -> pc.Seq[BenchmarkResult]:
    """Benchmark joinpath operation."""
    pyopath_base = pyopath.PurePath("/home/user")
    pathlib_base = pathlib.PurePath("/home/user")

    return _compare(
        "joinpath",
        lambda: pyopath_base.joinpath("documents", "file.txt"),
        lambda: pathlib_base.joinpath("documents", "file.txt"),
        iterations=100000,
    )


def benchmark_parent_chain() -> pc.Seq[BenchmarkResult]:
    """Benchmark traversing parent chain."""
    pyopath_p = pyopath.PurePath("/a/b/c/d/e/f/g/h/i/j")
    pathlib_p = pathlib.PurePath("/a/b/c/d/e/f/g/h/i/j")

    return _compare(
        "parents traversal",
        lambda: list(pyopath_p.parents),
        lambda: list(pathlib_p.parents),
        iterations=50000,
    )


def benchmark_with_suffix() -> pc.Seq[BenchmarkResult]:
    """Benchmark with_suffix operation."""
    pyopath_p = pyopath.PurePath("/home/user/file.txt")
    pathlib_p = pathlib.PurePath("/home/user/file.txt")

    return _compare(
        "with_suffix",
        lambda: pyopath_p.with_suffix(".md"),
        lambda: pathlib_p.with_suffix(".md"),
        iterations=100000,
    )


def benchmark_is_absolute() -> pc.Seq[BenchmarkResult]:
    """Benchmark is_absolute check."""
    pyopath_p = pyopath.PurePath("/home/user/file.txt")
    pathlib_p = pathlib.PurePath("/home/user/file.txt")

    return _compare(
        "is_absolute",
        lambda: pyopath_p.is_absolute(),
        lambda: pathlib_p.is_absolute(),
        iterations=100000,
    )


def benchmark_file_read(tmp_path: str) -> pc.Seq[BenchmarkResult]:
    """Benchmark file reading."""
    pyopath_p = pyopath.Path(tmp_path).joinpath("bench_read.txt")
    pathlib_p = pathlib.Path(tmp_path).joinpath("bench_read.txt")
    pathlib_p.write_text("x" * 1000)

    return _compare(
        "read_text (1KB)",
        lambda: pyopath_p.read_text(),
        lambda: pathlib_p.read_text(),
        iterations=10000,
    )


def benchmark_file_write(tmp_path: str) -> pc.Seq[BenchmarkResult]:
    """Benchmark file writing."""
    pyopath_p = pyopath.Path(tmp_path).joinpath("bench_write_pyopath.txt")
    pathlib_p = pathlib.Path(tmp_path).joinpath("bench_write_pathlib.txt")
    content = "x" * 1000

    return _compare(
        "write_text (1KB)",
        lambda: pyopath_p.write_text(content),
        lambda: pathlib_p.write_text(content),
        iterations=5000,
    )


def benchmark_exists(tmp_path: str) -> pc.Seq[BenchmarkResult]:
    """Benchmark exists check."""
    pyopath_p = pyopath.Path(tmp_path)
    pathlib_p = pathlib.Path(tmp_path)

    return _compare(
        "exists",
        lambda: pyopath_p.exists(),
        lambda: pathlib_p.exists(),
        iterations=50000,
    )


def benchmark_glob(tmp_path: str) -> pc.Seq[BenchmarkResult]:
    """Benchmark glob operation."""
    pyopath_p = pyopath.Path(tmp_path)
    pathlib_p = pathlib.Path(tmp_path)

    # Create some files
    pc.Iter.from_count().take(20).for_each(
        lambda i: pathlib.Path(tmp_path).joinpath(f"file_{i}.txt").touch()
    )

    return _compare(
        "glob *.txt",
        lambda: list(pyopath_p.glob("*.txt")),
        lambda: list(pathlib_p.glob("*.txt")),
        iterations=5000,
    )


def benchmark_iterdir(tmp_path: str) -> pc.Seq[BenchmarkResult]:
    """Benchmark iterdir operation."""
    pyopath_p = pyopath.Path(tmp_path)
    pathlib_p = pathlib.Path(tmp_path)

    return _compare(
        "iterdir",
        lambda: list(pyopath_p.iterdir()),
        lambda: list(pathlib_p.iterdir()),
        iterations=10000,
    )


def _print_results(results: pc.Seq[BenchmarkResult]) -> None:
    """Print benchmark results with comparison."""
    pyopath_result = results.nth(0)
    pathlib_result = results.nth(1)

    speedup = pathlib_result.avg_time_us / pyopath_result.avg_time_us
    faster = "pyopath" if speedup > 1 else "pathlib"
    ratio = speedup if speedup > 1 else 1 / speedup

    print(f"\n{pyopath_result.name}:")
    print(f"  {pyopath_result}")
    print(f"  {pathlib_result}")
    print(f"  → {faster} is {ratio:.2f}x faster")


def main() -> None:
    """Run all benchmarks."""
    print("=" * 70)
    print("PYOPATH VS PATHLIB BENCHMARK")
    print("=" * 70)

    # Pure path benchmarks (no filesystem)
    print("\n" + "-" * 70)
    print("PURE PATH OPERATIONS (no filesystem)")
    print("-" * 70)

    pc.Seq(
        (
            benchmark_pure_path_creation(),
            benchmark_path_parts(),
            benchmark_joinpath(),
            benchmark_parent_chain(),
            benchmark_with_suffix(),
            benchmark_is_absolute(),
        )
    ).iter().for_each(_print_results)

    # Filesystem benchmarks
    print("\n" + "-" * 70)
    print("FILESYSTEM OPERATIONS")
    print("-" * 70)

    with tempfile.TemporaryDirectory() as tmp:
        pc.Seq(
            (
                benchmark_exists(tmp),
                benchmark_file_read(tmp),
                benchmark_file_write(tmp),
                benchmark_glob(tmp),
                benchmark_iterdir(tmp),
            )
        ).iter().for_each(_print_results)

    print("\n" + "=" * 70)
    print("BENCHMARK COMPLETE")
    print("=" * 70)


if __name__ == "__main__":
    main()
