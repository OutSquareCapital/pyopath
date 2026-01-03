"""Check that all pathlib functions have a pyopath equivalent."""

import pathlib
from dataclasses import dataclass
from enum import StrEnum, auto
from pathlib import Path
from typing import Literal, Self

import polars as pl
import pyochain as pc
import pyopath

DATA = Path("scripts", "data")


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


class Library(StrEnum):
    """Enumeration of source libraries."""

    PATHLIB = auto()
    PYOPATH = auto()
    DEFAULT = auto()


class MemberType(StrEnum):
    """Enumeration of member types."""

    METHOD = auto()
    STATICMETHOD = auto()
    CLASSMETHOD = auto()
    PROPERTY = auto()
    DEFAULT = auto()


DF_SCHEMA = {
    "source": pl.Enum(Library),
    "class": pl.Enum(_pyopath_classes().iter().map(lambda x: x.__name__).sort()),
    "member": pl.String,
    "type": pl.Enum(MemberType),
}


@dataclass(slots=True)
class Member:
    """Represents a class member with its source library."""

    source: Library = Library.DEFAULT
    class_name: str = ""
    name: str = ""
    member_type: MemberType = MemberType.DEFAULT

    def with_type(self, obj: object) -> Self:
        """Return a Member with the given type based on the object."""
        type_name = type(obj).__name__
        if isinstance(obj, property) or type_name == "getset_descriptor":
            self.member_type = MemberType.PROPERTY
        elif isinstance(obj, staticmethod):
            self.member_type = MemberType.STATICMETHOD
        elif isinstance(obj, classmethod):
            self.member_type = MemberType.CLASSMETHOD
        else:
            self.member_type = MemberType.METHOD
        return self

    def with_source(self, src: Library) -> Self:
        """Return a Member with the given source library."""
        self.source = src
        return self


def _extract_members(cls: type) -> pc.Iter[Member]:
    def _is_callable_or_property(obj: object) -> bool:
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
        .map(lambda kv: Member(class_name=cls.__name__, name=kv[0]).with_type(kv[1]))
    )


def _save_and_report(df: pl.LazyFrame) -> None:
    def _show_if_exist(df: pl.DataFrame, kword: Literal["MISSING", "EXTRA"]) -> None:
        if df.height > 0:
            print("\n" + "-" * 60)
            print(f"{kword} IN PYOPATH (not in pathlib):")
            print("-" * 60)
            return (
                pc.Iter(df.iter_rows(named=True))
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
        return None

    # Find and save missing members (in pathlib but not pyopath)
    missing_df = _extra_or_missing(df.clone(), Library.PATHLIB, Library.PYOPATH)
    missing_df.clone().sink_ndjson(DATA.joinpath("missing_in_pyopath.ndjson"))

    # Find and save extra members (in pyopath but not pathlib)
    extra_df = _extra_or_missing(df.clone(), Library.PYOPATH, Library.PATHLIB)
    extra_df.clone().sink_ndjson(DATA.joinpath("extra_in_pyopath.ndjson"))
    print("=" * 60)
    print("PYOPATH COMPATIBILITY REPORT")
    print("=" * 60)
    missing_df.collect().pipe(_show_if_exist, "MISSING")
    extra_df.collect().pipe(_show_if_exist, "EXTRA")


def _extra_or_missing(
    df: pl.LazyFrame, is_in: Library, is_not_in: Library
) -> pl.LazyFrame:
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
    return (
        _pathlib_classes()
        .iter()
        .flat_map(
            lambda cls: _extract_members(cls).map(
                lambda t: t.with_source(Library.PATHLIB)
            )
        )
        .chain(
            _pyopath_classes()
            .iter()
            .flat_map(
                lambda cls: _extract_members(cls).map(
                    lambda t: t.with_source(Library.PYOPATH)
                )
            )
        )
        .into(lambda x: pl.LazyFrame(x, schema=DF_SCHEMA))
        .filter(pl.col("member").str.starts_with("_").not_())
        .unique(subset=["source", "class", "member"])
        .sort(["class", "member", "source"])
        .pipe(_save_and_report)
    )


if __name__ == "__main__":
    main()
