"""Check that all pathlib functions have a pyopath equivalent."""

import pathlib
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, Self

import polars as pl
import pyochain as pc
import pyopath

DATA = Path("scripts", "data")
Library = Literal["pathlib", "pyopath", "default"]
MemberType = Literal["method", "staticmethod", "classmethod", "property", "default"]


@dataclass(slots=True)
class Member:
    """Represents a class member with its source library."""

    class_name: str
    name: str
    member_type: MemberType = "default"
    source: Library = "default"

    def with_type(self, obj: object) -> Self:
        """Return a Member with the given type based on the object."""
        type_name = type(obj).__name__
        if isinstance(obj, property) or type_name == "getset_descriptor":
            self.member_type = "property"
        if isinstance(obj, staticmethod):
            self.member_type = "staticmethod"
        if isinstance(obj, classmethod):
            self.member_type = "classmethod"
        self.member_type = "method"
        return self

    def with_source(self, src: Library) -> Self:
        """Return a Member with the given source library."""
        self.source = src
        return self


def _pathlib_classes() -> pc.Set[type]:
    """Return all pathlib classes to check."""
    return pc.Set(
        (
            pathlib.PurePath,
            pathlib.PurePosixPath,
            pathlib.PureWindowsPath,
            pathlib.Path,
            pathlib.PosixPath,
            pathlib.WindowsPath,
        )
    )


def _pyopath_classes() -> pc.Set[type]:
    """Return all pyopath classes to check."""
    return pc.Set(
        (
            pyopath.PurePath,
            pyopath.PurePosixPath,
            pyopath.PureWindowsPath,
            pyopath.Path,
            pyopath.PosixPath,
            pyopath.WindowsPath,
        )
    )


def build_comparison_df(
    pathlib_classes: pc.Set[type], pyopath_classes: pc.Set[type]
) -> pl.LazyFrame:
    """Build a LazyFrame comparing pathlib and pyopath members."""
    member: pl.Expr = pl.col("member")

    return (
        pathlib_classes.iter()
        .flat_map(
            lambda cls: _extract_members(cls).map(lambda t: t.with_source("pathlib"))
        )
        .chain(
            pyopath_classes.iter().flat_map(
                lambda cls: _extract_members(cls).map(
                    lambda t: t.with_source("pyopath")
                )
            )
        )
        .into(lambda x: pl.LazyFrame(x, schema=["source", "class", "member", "type"]))
        .filter(member.str.starts_with("_").not_())
        .unique(subset=["source", "class", "member"])
        .sort(["class", "member", "source"])
    )


def _extract_members(cls: type) -> pc.Iter[Member]:
    """Extract all public members from a class with their type."""

    def _is_callable_or_property(obj: object) -> bool:
        """Check if object is callable, staticmethod, classmethod, property, or PyO3 descriptor."""
        return (
            callable(obj)
            or isinstance(obj, (staticmethod, classmethod, property))
            or type(obj).__name__ in ("getset_descriptor", "method_descriptor")
        )

    return (
        pc.Iter(cls.mro())
        .take_while(lambda x: x is not object)
        .flat_map(lambda x: x.__dict__.items())
        .filter(lambda kv: _is_callable_or_property(kv[1]))
        .map(lambda kv: Member(cls.__name__, kv[0]).with_type(kv[1]))
    )


def _extra_or_missing(
    df: pl.LazyFrame, is_in: Library, is_not_in: Library
) -> pl.LazyFrame:
    """Find members that exist in pathlib but not in pyopath."""
    return (
        df.group_by("class", "member")
        .agg(
            pl.col("source").alias("sources"),
            pl.col("type").first().alias("type"),
        )
        .filter(
            pl.col("sources")
            .list.contains(is_in)
            .and_(pl.col("sources").list.contains(is_not_in).not_())
        )
        .select("class", "member", "type")
        .sort(["class", "member"])
    )


def main() -> None:
    """Run the comparison and output the results."""
    DATA.mkdir(parents=True, exist_ok=True)

    comparison_df = build_comparison_df(_pathlib_classes(), _pyopath_classes())

    # Find and save missing members (in pathlib but not pyopath)
    missing_df = _extra_or_missing(comparison_df.clone(), "pathlib", "pyopath")
    missing_df.clone().sink_ndjson(DATA.joinpath("missing_in_pyopath.ndjson"))

    # Find and save extra members (in pyopath but not pathlib)
    extra_df = _extra_or_missing(comparison_df.clone(), "pyopath", "pathlib")
    extra_df.clone().sink_ndjson(DATA.joinpath("extra_in_pyopath.ndjson"))

    # Print summary
    missing_collected = missing_df.collect()
    extra_collected = extra_df.collect()

    print("=" * 60)
    print("PYOPATH COMPATIBILITY REPORT")
    print("=" * 60)
    print(f"❌ Missing:     {missing_collected.height} members")
    print(f"++ Extra:       {extra_collected.height} members")

    if missing_collected.height > 0:
        print("\n" + "-" * 60)
        print("MISSING IN PYOPATH (by class):")
        print("-" * 60)
        (
            pc.Iter(missing_collected.iter_rows(named=True))
            .group_by(lambda x: x["class"])
            .for_each(
                lambda group: print(
                    f"\n{group.key}:\n"
                    + group.values.map(
                        lambda x: f"  - {x['member']} ({x['type']})"
                    ).join("\n")
                )
            )
        )

    if extra_collected.height > 0:
        print("\n" + "-" * 60)
        print("EXTRA IN PYOPATH (not in pathlib):")
        print("-" * 60)
        (
            pc.Iter(extra_collected.iter_rows(named=True))
            .group_by(lambda x: x["class"])
            .for_each(
                lambda group: print(
                    f"\n{group.key}:\n"
                    + group.values.map(
                        lambda x: f"  - {x['member']} ({x['type']})"
                    ).join("\n")
                )
            )
        )

    print("\n" + "=" * 60)
    print(f"Files written to: {DATA.resolve()}")
    print("=" * 60)


if __name__ == "__main__":
    main()
