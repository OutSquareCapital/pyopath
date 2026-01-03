"""Check 1:1 correspondence of function signatures between pathlib and pyopath.

This script compares:
1. Method presence
2. Signature parameters: names, defaults, keyword-only
"""

import inspect
import json
import pathlib
from dataclasses import dataclass
from pathlib import Path

import pyopath

DATA = Path("scripts", "data")


def _pathlib_classes() -> dict[str, type]:
    """Return all pathlib classes to check."""
    return {
        "PurePath": pathlib.PurePath,
        "PurePosixPath": pathlib.PurePosixPath,
        "PureWindowsPath": pathlib.PureWindowsPath,
        "Path": pathlib.Path,
        "PosixPath": pathlib.PosixPath,
        "WindowsPath": pathlib.WindowsPath,
    }


def _pyopath_classes() -> dict[str, type]:
    """Return all pyopath classes to check."""
    return {
        "PurePath": pyopath.PurePath,
        "PurePosixPath": pyopath.PurePosixPath,
        "PureWindowsPath": pyopath.PureWindowsPath,
        "Path": pyopath.Path,
        "PosixPath": pyopath.PosixPath,
        "WindowsPath": pyopath.WindowsPath,
    }


@dataclass(slots=True)
class ParamInfo:
    """Information about a function parameter."""

    name: str
    kind: str
    has_default: bool
    default_value: str | None

    def to_str(self) -> str:
        """Return string representation."""
        kind_prefix = ""
        if self.kind == "VAR_POSITIONAL":
            kind_prefix = "*"
        elif self.kind == "VAR_KEYWORD":
            kind_prefix = "**"

        default_suffix = ""
        if self.has_default:
            default_suffix = f"={self.default_value}"

        return f"{kind_prefix}{self.name}{default_suffix}"


@dataclass(slots=True)
class SignatureInfo:
    """Information about a method signature."""

    class_name: str
    method_name: str
    params: list[ParamInfo]
    is_property: bool
    is_classmethod: bool
    is_staticmethod: bool

    def params_str(self) -> str:
        """Return string representation of parameters."""
        return ", ".join(p.to_str() for p in self.params)


def _extract_signature(
    cls: type, method_name: str, obj: object
) -> SignatureInfo | None:
    """Extract signature information from a method."""
    if method_name.startswith("_"):
        return None

    is_property = isinstance(obj, property) or type(obj).__name__ == "getset_descriptor"
    is_classmethod = isinstance(obj, classmethod)
    is_staticmethod = isinstance(obj, staticmethod)

    if is_property:
        return SignatureInfo(
            class_name=cls.__name__,
            method_name=method_name,
            params=[],
            is_property=True,
            is_classmethod=False,
            is_staticmethod=False,
        )

    actual_func = obj
    if is_classmethod or is_staticmethod:
        actual_func = obj.__func__  # type: ignore[union-attr]

    try:
        sig = inspect.signature(actual_func)  # type: ignore[arg-type]
    except (ValueError, TypeError):
        return None

    params: list[ParamInfo] = []
    for param in sig.parameters.values():
        if param.name in ("self", "cls"):
            continue
        has_default = param.default is not inspect.Parameter.empty
        default_str = repr(param.default) if has_default else None
        params.append(
            ParamInfo(
                name=param.name,
                kind=param.kind.name,
                has_default=has_default,
                default_value=default_str,
            )
        )

    return SignatureInfo(
        class_name=cls.__name__,
        method_name=method_name,
        params=params,
        is_property=is_property,
        is_classmethod=is_classmethod,
        is_staticmethod=is_staticmethod,
    )


def _get_all_signatures(
    classes: dict[str, type],
) -> dict[tuple[str, str], SignatureInfo]:
    """Get all method signatures from a dict of classes."""
    result: dict[tuple[str, str], SignatureInfo] = {}

    for class_name, cls in classes.items():
        for name in dir(cls):
            if name.startswith("_") and not name.startswith("__"):
                continue
            if (
                name.startswith("__")
                and name.endswith("__")
                and name
                not in (
                    "__fspath__",
                    "__truediv__",
                    "__rtruediv__",
                )
            ):
                continue

            try:
                obj = getattr(cls, name)
            except AttributeError:
                continue

            sig = _extract_signature(cls, name, obj)
            if sig is not None:
                result[(class_name, name)] = sig

    return result


def _compare_signatures(
    pathlib_sigs: dict[tuple[str, str], SignatureInfo],
    pyopath_sigs: dict[tuple[str, str], SignatureInfo],
) -> list[dict[str, object]]:
    """Compare signatures and return differences."""
    differences: list[dict[str, object]] = []

    for key, pathlib_sig in pathlib_sigs.items():
        class_name, method_name = key

        pyopath_sig = pyopath_sigs.get(key)
        if pyopath_sig is None:
            differences.append(
                {
                    "class": class_name,
                    "method": method_name,
                    "issue": "MISSING",
                    "pathlib": pathlib_sig.params_str(),
                    "pyopath": "N/A",
                }
            )
            continue

        if pathlib_sig.is_property != pyopath_sig.is_property:
            differences.append(
                {
                    "class": class_name,
                    "method": method_name,
                    "issue": "TYPE_MISMATCH",
                    "pathlib": "property" if pathlib_sig.is_property else "method",
                    "pyopath": "property" if pyopath_sig.is_property else "method",
                }
            )
            continue

        if pathlib_sig.is_property:
            continue

        pathlib_params = pathlib_sig.params_str()
        pyopath_params = pyopath_sig.params_str()

        if pathlib_params != pyopath_params:
            differences.append(
                {
                    "class": class_name,
                    "method": method_name,
                    "issue": "SIGNATURE_MISMATCH",
                    "pathlib": pathlib_params,
                    "pyopath": pyopath_params,
                }
            )

    return differences


def _print_report(differences: list[dict[str, object]]) -> None:
    """Print a formatted report of differences."""
    if not differences:
        print("✅ All signatures match!")
        return

    print("=" * 80)
    print("SIGNATURE COMPARISON REPORT")
    print("=" * 80)

    grouped: dict[str, list[dict[str, object]]] = {}
    for diff in differences:
        issue = str(diff["issue"])
        if issue not in grouped:
            grouped[issue] = []
        grouped[issue].append(diff)

    for issue_type, diffs in grouped.items():
        print(f"\n{'─' * 80}")
        print(f"📌 {issue_type} ({len(diffs)} issues)")
        print("─" * 80)

        by_class: dict[str, list[dict[str, object]]] = {}
        for diff in diffs:
            class_name = str(diff["class"])
            if class_name not in by_class:
                by_class[class_name] = []
            by_class[class_name].append(diff)

        for class_name, class_diffs in by_class.items():
            print(f"\n  {class_name}:")
            for diff in class_diffs:
                method = diff["method"]
                pathlib_val = diff["pathlib"]
                pyopath_val = diff["pyopath"]

                if issue_type == "MISSING":
                    print(f"    ❌ {method}({pathlib_val})")
                elif issue_type == "TYPE_MISMATCH":
                    print(
                        f"    ⚠️  {method}: pathlib={pathlib_val}, pyopath={pyopath_val}"
                    )
                elif issue_type == "SIGNATURE_MISMATCH":
                    print(f"    🔧 {method}")
                    print(f"       pathlib: ({pathlib_val})")
                    print(f"       pyopath: ({pyopath_val})")


def main() -> None:
    """Run signature comparison."""
    DATA.mkdir(parents=True, exist_ok=True)

    print("Extracting pathlib signatures...")
    pathlib_sigs = _get_all_signatures(_pathlib_classes())
    print(f"  Found {len(pathlib_sigs)} signatures")

    print("Extracting pyopath signatures...")
    pyopath_sigs = _get_all_signatures(_pyopath_classes())
    print(f"  Found {len(pyopath_sigs)} signatures")

    print("\nComparing signatures...")
    differences = _compare_signatures(pathlib_sigs, pyopath_sigs)

    _print_report(differences)

    if differences:
        output_file = DATA.joinpath("signature_differences.ndjson")
        with output_file.open("w") as f:
            for diff in differences:
                f.write(json.dumps(diff) + "\n")
        print(f"\n📄 Saved {len(differences)} differences to {output_file}")


if __name__ == "__main__":
    main()
