"""Check 1:1 correspondence of function signatures between pathlib and pyopath.

Compares:
1. Method presence
2. Signature parameters: names, defaults, keyword-only
3. Return type annotations (from .pyi stubs)
"""

import ast
import json
from dataclasses import dataclass
from pathlib import Path

import pyochain as pc

DATA = Path("scripts", "data")

# Paths to .pyi stub files
PATHLIB_STUB = Path("reference", "__init__.pyi")

PYOPATH_STUB = Path("pyopath.pyi")

# Class hierarchy for inheritance resolution
CLASS_HIERARCHY: pc.Dict[str, pc.Seq[str]] = pc.Dict.from_kwargs(
    PurePath=pc.Seq(()),
    PurePosixPath=pc.Seq(("PurePath",)),
    PureWindowsPath=pc.Seq(("PurePath",)),
    Path=pc.Seq(("PurePath",)),
    PosixPath=pc.Seq(("Path", "PurePosixPath", "PurePath")),
    WindowsPath=pc.Seq(("Path", "PureWindowsPath", "PurePath")),
)

TARGET_CLASSES = pc.Set(
    ("PurePath", "PurePosixPath", "PureWindowsPath", "Path", "PosixPath", "WindowsPath")
)


@dataclass(slots=True)
class ParamInfo:
    """Information about a function parameter."""

    name: str
    kind: str
    has_default: bool
    default_repr: pc.Option[str]

    def to_str(self) -> str:
        """Return string representation."""
        prefix = (
            pc.Some("*")
            if self.kind == "var_positional"
            else pc.Some("**")
            if self.kind == "var_keyword"
            else pc.NONE
        ).unwrap_or("")
        suffix = self.default_repr.map(lambda v: f"={v}").unwrap_or("")
        return f"{prefix}{self.name}{suffix}"


@dataclass(slots=True)
class SignatureInfo:
    """Information about a method signature."""

    class_name: str
    method_name: str
    params: pc.Seq[ParamInfo]
    return_type: str
    is_property: bool
    is_classmethod: bool
    is_staticmethod: bool

    def params_str(self) -> str:
        """Return string representation of parameters."""
        return self.params.iter().map(lambda p: p.to_str()).join(", ")

    def full_sig(self) -> str:
        """Return full signature string."""
        return f"({self.params_str()}) -> {self.return_type}"


@dataclass(slots=True)
class Difference:
    """A difference between pathlib and pyopath."""

    class_name: str
    method_name: str
    issue: str
    pathlib_val: str
    pyopath_val: str

    def to_dict(self) -> pc.Dict[str, str]:
        """Convert to dict for JSON serialization."""
        return pc.Dict.from_kwargs(
            class_name=self.class_name,
            method=self.method_name,
            issue=self.issue,
            pathlib=self.pathlib_val,
            pyopath=self.pyopath_val,
        )


def _ast_to_type_str(node: ast.expr | None) -> str:
    """Convert an AST node to a type string."""
    if node is None:
        return "None"

    match node:
        case ast.Constant(value=v):
            return repr(v) if isinstance(v, str) else str(v)
        case ast.Name(id=name):
            return name
        case ast.Attribute(value=val, attr=attr):
            return f"{_ast_to_type_str(val)}.{attr}"
        case ast.Subscript(value=val, slice=sl):
            return f"{_ast_to_type_str(val)}[{_ast_to_type_str(sl)}]"
        case ast.Tuple(elts=elts):
            return pc.Iter(elts).map(_ast_to_type_str).join(", ")
        case ast.BinOp(left=left, op=ast.BitOr(), right=right):
            return f"{_ast_to_type_str(left)} | {_ast_to_type_str(right)}"
        case ast.List(elts=elts):
            inner = pc.Iter(elts).map(_ast_to_type_str).join(", ")
            return f"[{inner}]"
        case _:
            return ast.unparse(node)


def _extract_param(arg: ast.arg, default: ast.expr | None, kind: str) -> ParamInfo:
    """Extract parameter info from AST arg node."""
    return ParamInfo(
        name=arg.arg,
        kind=kind,
        has_default=default is not None,
        default_repr=pc.Option.from_(default).map(_ast_to_type_str),
    )


def _extract_function_signature(
    class_name: str,
    func: ast.FunctionDef | ast.AsyncFunctionDef,
    is_property: bool = False,
) -> SignatureInfo:
    """Extract signature info from AST function definition."""
    args = func.args
    params = pc.Vec[ParamInfo].new()

    is_classmethod = pc.Iter(func.decorator_list).any(
        lambda d: isinstance(d, ast.Name) and d.id == "classmethod"
    )
    is_staticmethod = pc.Iter(func.decorator_list).any(
        lambda d: isinstance(d, ast.Name) and d.id == "staticmethod"
    )

    num_positional = len(args.posonlyargs) + len(args.args)
    num_defaults = len(args.defaults)
    defaults_start = num_positional - num_defaults

    # Positional-only args
    pc.Iter(args.posonlyargs).enumerate().for_each(
        lambda idx_arg: params.append(
            _extract_param(
                idx_arg[1],
                args.defaults[idx_arg[0] - defaults_start]
                if idx_arg[0] >= defaults_start
                else None,
                "positional_only",
            )
        )
    )

    # Regular args (skip self/cls)
    pc.Iter(args.args).enumerate().filter(
        lambda idx_arg: idx_arg[1].arg not in {"self", "cls"}
    ).for_each(
        lambda idx_arg: params.append(
            _extract_param(
                idx_arg[1],
                args.defaults[len(args.posonlyargs) + idx_arg[0] - defaults_start]
                if len(args.posonlyargs) + idx_arg[0] >= defaults_start
                else None,
                "positional_or_keyword",
            )
        )
    )

    # *args
    pc.Option.from_(args.vararg).inspect(
        lambda va: params.append(
            ParamInfo(
                name=va.arg,
                kind="var_positional",
                has_default=False,
                default_repr=pc.NONE,
            )
        )
    )

    # Keyword-only args
    pc.Iter(args.kwonlyargs).zip(args.kw_defaults, strict=False).for_each(
        lambda arg_def: params.append(
            _extract_param(arg_def[0], arg_def[1], "keyword_only")
        )
    )

    # **kwargs
    pc.Option.from_(args.kwarg).inspect(
        lambda kw: params.append(
            ParamInfo(
                name=kw.arg, kind="var_keyword", has_default=False, default_repr=pc.NONE
            )
        )
    )

    return SignatureInfo(
        class_name=class_name,
        method_name=func.name,
        params=params.into(pc.Seq),
        return_type=_ast_to_type_str(func.returns),
        is_property=is_property,
        is_classmethod=is_classmethod,
        is_staticmethod=is_staticmethod,
    )


def _should_include_method(name: str) -> bool:
    """Check if a method should be included in comparison."""
    if name.startswith("_") and not name.startswith("__"):
        return False
    if name.startswith("__") and name.endswith("__"):
        return name in ("__fspath__", "__truediv__", "__rtruediv__")
    return True


def _parse_stub_file(stub_path: Path) -> pc.Dict[str, pc.Dict[str, SignatureInfo]]:
    """Parse a .pyi stub file and extract all method signatures per class."""
    content = stub_path.read_text(encoding="utf-8")
    tree = ast.parse(content)

    result = pc.Dict[str, pc.Dict[str, SignatureInfo]].new()

    def _process_class(cls: ast.ClassDef) -> None:
        if cls.name not in TARGET_CLASSES:
            return

        methods = pc.Dict[str, SignatureInfo].new()
        seen_overloads = pc.SetMut[str](())

        for item in cls.body:
            match item:
                case ast.FunctionDef() | ast.AsyncFunctionDef() if (
                    _should_include_method(item.name)
                ):
                    is_prop = pc.Iter(item.decorator_list).any(
                        lambda d: isinstance(d, ast.Name) and d.id == "property"
                    )
                    is_overload = pc.Iter(item.decorator_list).any(
                        lambda d: isinstance(d, ast.Name) and d.id == "overload"
                    )

                    # Skip subsequent overloads
                    if is_overload:
                        if item.name in seen_overloads:
                            continue
                        seen_overloads.add(item.name)

                    sig = _extract_function_signature(cls.name, item, is_prop)
                    methods[item.name] = sig
                case _:
                    pass

        result[cls.name] = methods

    pc.Iter(tree.body).filter(lambda n: isinstance(n, ast.ClassDef)).for_each(
        lambda n: _process_class(n)  # type: ignore[arg-type]
    )

    return result


def _resolve_inheritance(
    class_sigs: pc.Dict[str, pc.Dict[str, SignatureInfo]],
) -> pc.Dict[tuple[str, str], SignatureInfo]:
    """Resolve inheritance and return flattened signatures."""
    result = pc.Dict[tuple[str, str], SignatureInfo].new()

    for class_name in TARGET_CLASSES:
        # Get all parent classes
        parents = CLASS_HIERARCHY.get_item(class_name).unwrap_or(pc.Seq(()))

        # Collect methods from parents first, then override with own methods
        methods = pc.Dict[str, SignatureInfo].new()

        # Add parent methods (in reverse order so closer parents override)
        parents.iter().rev().for_each(
            lambda parent: class_sigs.get_item(parent).inspect(
                lambda parent_methods: parent_methods.iter().for_each(
                    lambda kv: methods.insert(kv.key, kv.value)
                )
            )
        )

        # Add own methods (override parents)
        class_sigs.get_item(class_name).inspect(
            lambda own_methods: own_methods.iter().for_each(
                lambda kv: methods.insert(kv.key, kv.value)
            )
        )

        # Add to result with class_name in key
        methods.iter().for_each(
            lambda kv: result.insert(
                (class_name, kv.key),
                SignatureInfo(
                    class_name=class_name,
                    method_name=kv.value.method_name,
                    params=kv.value.params,
                    return_type=kv.value.return_type,
                    is_property=kv.value.is_property,
                    is_classmethod=kv.value.is_classmethod,
                    is_staticmethod=kv.value.is_staticmethod,
                ),
            )
        )

    return result


def _normalize_return_type(ret_type: str) -> str:
    """Normalize return type for comparison."""
    if ret_type in ("Self", "PurePath", "Path", "PosixPath", "WindowsPath"):
        return "Self"

    normalized = ret_type.replace("Generator", "Iterator")
    if "Sequence" in normalized or "tuple" in normalized:
        return normalized.replace("tuple", "Sequence")

    return normalized


def _compare_signatures(
    pathlib_sigs: pc.Dict[tuple[str, str], SignatureInfo],
    pyopath_sigs: pc.Dict[tuple[str, str], SignatureInfo],
) -> pc.Vec[Difference]:
    """Compare signatures and return differences."""

    def _compare_one(
        key: tuple[str, str], pathlib_sig: SignatureInfo
    ) -> pc.Option[Difference]:
        class_name, method_name = key

        return (
            pyopath_sigs.get_item(key)
            .map(
                lambda pyopath_sig: _check_differences(
                    class_name, method_name, pathlib_sig, pyopath_sig
                )
            )
            .unwrap_or_else(
                lambda: pc.Some(
                    Difference(
                        class_name=class_name,
                        method_name=method_name,
                        issue="MISSING",
                        pathlib_val=pathlib_sig.full_sig(),
                        pyopath_val="N/A",
                    )
                )
            )
        )

    def _check_differences(
        class_name: str,
        method_name: str,
        pathlib_sig: SignatureInfo,
        pyopath_sig: SignatureInfo,
    ) -> pc.Option[Difference]:
        if pathlib_sig.is_property != pyopath_sig.is_property:
            return pc.Some(
                Difference(
                    class_name=class_name,
                    method_name=method_name,
                    issue="TYPE_MISMATCH",
                    pathlib_val="property" if pathlib_sig.is_property else "method",
                    pyopath_val="property" if pyopath_sig.is_property else "method",
                )
            )

        if pathlib_sig.is_property:
            return pc.NONE

        pathlib_params = pathlib_sig.params_str()
        pyopath_params = pyopath_sig.params_str()

        if pathlib_params != pyopath_params:
            return pc.Some(
                Difference(
                    class_name=class_name,
                    method_name=method_name,
                    issue="SIGNATURE_MISMATCH",
                    pathlib_val=pathlib_sig.full_sig(),
                    pyopath_val=pyopath_sig.full_sig(),
                )
            )

        pathlib_ret = _normalize_return_type(pathlib_sig.return_type)
        pyopath_ret = _normalize_return_type(pyopath_sig.return_type)

        if pathlib_ret != pyopath_ret:
            return pc.Some(
                Difference(
                    class_name=class_name,
                    method_name=method_name,
                    issue="RETURN_TYPE_MISMATCH",
                    pathlib_val=pathlib_sig.return_type,
                    pyopath_val=pyopath_sig.return_type,
                )
            )

        return pc.NONE

    return (
        pathlib_sigs.iter()
        .filter_map(lambda kv: _compare_one(kv[0], kv[1]))
        .collect(pc.Vec)
    )


def main() -> None:
    """Run signature comparison."""
    DATA.mkdir(parents=True, exist_ok=True)

    pathlib_sigs = _parse_stub_file(PATHLIB_STUB).into(_resolve_inheritance)
    pyopath_sigs = _parse_stub_file(PYOPATH_STUB).into(_resolve_inheritance)

    output_file = DATA.joinpath("signature_differences.ndjson")
    with output_file.open("w") as f:
        _compare_signatures(pathlib_sigs, pyopath_sigs).iter().for_each(
            lambda diff: f.write(json.dumps(dict(diff.to_dict())) + "\n")
        )


if __name__ == "__main__":
    main()
