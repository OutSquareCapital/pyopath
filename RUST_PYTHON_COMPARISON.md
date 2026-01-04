# Analyse de Correspondance Rust ↔ Python pour PyOPath

## 🎯 Objectif

Établir une correspondance 1:1 précise entre l'implémentation Rust et la référence Python de `PurePath`.

---

## 📊 Vue d'ensemble des structures

### Python (`reference/__init__.py`)

```python
class PurePath:
    __slots__ = (
        "_raw_paths",      # Liste de chemins non joints
        "_drv",            # Drive (C:, \\server\share, etc.)
        "_root",           # Root (/, \, etc.)
        "_tail_cached",    # Liste des parties du chemin
        "_str",            # Représentation string complète
        "_str_normcase_cached",   # String normalisé (lowercase sur Windows)
        "_parts_normcase_cached", # Parts normalisés pour comparaisons
        "_hash",           # Hash du chemin normalisé
    )
    parser = os.path  # posixpath ou ntpath
```

### Rust (`src/macros.rs` + `src/core.rs`)

```rust
// Dans macros.rs
struct PurePosixPath {  // ou PureWindowsPath
    _raw_path_tuple: Vec<String>,           // ≈ _raw_paths
    str_repr_cached: OnceLock<String>,      // ≈ _str
    str_repr_original_cached: OnceLock<String>,  // NOUVEAU (pas en Python)
    parsed: OnceLock<ParsedParts>,          // NOUVEAU (structure groupée)
    _str_normcase_cached: OnceLock<String>, // ≈ _str_normcase_cached
    _parts_normcase_cached: OnceLock<Vec<String>>, // ≈ _parts_normcase_cached
}

// Dans core.rs
struct ParsedParts {
    drive: String,    // ≈ _drv
    root: String,     // ≈ _root
    parts: Vec<String>,  // ≈ _tail_cached
}
```

---

## 🔍 Différences Structurelles

### 1. ParsedParts vs champs individuels

| Python | Rust | Note |
|--------|------|------|
| `_drv: str` | `ParsedParts.drive: String` | ✅ Équivalent |
| `_root: str` | `ParsedParts.root: String` | ✅ Équivalent |
| `_tail_cached: list[str]` | `ParsedParts.parts: Vec<String>` | ✅ Équivalent |
| Stockés directement dans `PurePath` | Groupés dans `ParsedParts`, stockés dans `OnceLock` | ⚠️ **Design différent** |

**Implication**: En Rust, `_drv`, `_root`, `_tail` n'existent pas comme champs séparés. Ils sont tous calculés ensemble et stockés dans `ParsedParts`. Cela évite les calculs partiels.

### 2. Champs uniques à Rust

| Champ Rust | Équivalent Python | Raison |
|------------|-------------------|--------|
| `str_repr_original_cached`     | ❌ N'existe pas | Stocke le résultat d'`os.path.join()` AVANT normalisation des séparateurs |
| `parsed: OnceLock<ParsedParts>`| ❌ N'existe pas comme struct | Python calcule `_drv`, `_root`, `_tail` individuellement |

### 3. Absence en Rust

| Champ Python | Présent en Rust? | Impact |
|--------------|------------------|--------|
| `_hash: int` | ❌ NON | Rust recalcule le hash à chaque appel de `__hash__` |

---

## 🔧 Méthodes: Analyse Complète

### Méthodes de construction

| Python | Rust | Status | Notes |
|--------|------|--------|-------|
| `__new__(cls, *args)` | `#[new] fn new(py, args)` | ✅ | Logique identique |
| `__init__(self, *args)` | Intégré dans `new()` | ✅ | Rust combine `__new__` + `__init__` |
| `with_segments(*pathsegments)` | `with_segments(py, pathsegments)` | ✅ | Identique |
| `joinpath(*pathsegments)` | `joinpath(py, paths)` | ✅ | Identique |

### Méthodes internes (Python) vs helpers (Rust)

| Python | Rust | Status | Notes |
| -------- | ------ | -------- | ------- |
| `_from_parsed_parts(drv, root, tail)` | ❌ **MANQUANT** | ❌ | Créé un nouveau Path à partir de parties parsées |
| `_from_parsed_string(path_str)` | `from_str_repr(str_repr)` | ⚠️ | Nom différent, mais similaire |
| `_format_parsed_parts(drv, root, tail)` | ❌ **MANQUANT** | ❌ | Reconstruit un string à partir de parties |
| `_parse_path(path)` | `<Separator>::parse(raw_path)` | ⚠️ | Logique similaire mais en `separators.rs` |
| `_parse_pattern(pattern)` | ❌ **MANQUANT** | ❌ | Utilisé pour glob patterns |

**⚠️ PROBLÈME MAJEUR**:

- Python utilise `_from_parsed_parts()` partout (parent, with_name, relative_to, etc.)
- Rust utilise `from_str_repr()` qui reconstruit TOUT le string puis re-parse
- **Impact performance**: Rust fait plus de travail inutile

### Propriétés et getters

| Python | Rust | Status | Notes |
|--------|------|--------|-------|
| `drive` (property) | `#[getter] drive()` | ✅ | Identique |
| `root` (property) | `#[getter] root()` | ✅ | Identique |
| `anchor` (property) | `#[getter] anchor()` | ✅ | Identique |
| `parts` (property) | `#[getter] parts()` | ✅ | Retourne tuple en Python, PyTuple en Rust |
| `name` (property) | `#[getter] name()` | ✅ | Identique |
| `stem` (property) | `#[getter] stem()` | ✅ | Identique |
| `suffix` (property) | `#[getter] suffix()` | ✅ | Identique |
| `suffixes` (property) | `#[getter] suffixes()` | ✅ | Identique |
| `parent` (property) | `#[getter] parent()` | ✅ | Identique |
| `parents` (property) | `#[getter] parents()` | ✅ | Retourne `_PathParents` en Python, `PyList` en Rust |
| `_raw_path` (property) | ❌ **MANQUANT** | ❌ | Joint les `_raw_paths` en un seul string |
| `_tail` (property) | Via `parsed_parts().parts` | ⚠️ | Pas directement accessible |
| `_str_normcase` (property) | Via `str_normcase()` (méthode privée) | ⚠️ | En Rust c'est une méthode privée, pas un getter public |
| `_parts_normcase` (property) | Via `parts_normcase()` (méthode privée) | ⚠️ | Idem |

### Méthodes de transformation

| Python | Rust | Status | Notes |
|--------|------|--------|-------|
| `with_name(name)` | `with_name(py, name)` | ✅ | Logique en `separators.rs` |
| `with_stem(stem)` | `with_stem(py, stem)` | ✅ | Identique |
| `with_suffix(suffix)` | `with_suffix(py, suffix)` | ✅ | Logique en `separators.rs` |
| `as_posix()` | `as_posix()` | ✅ | Identique |
| `as_uri()` | `as_uri()` | ⚠️ | Rust n'a pas les warnings de deprecation |
| `__bytes__()` | `__bytes__(py)` | ✅ | Identique |

### Méthodes de comparaison et paths relatifs

| Python | Rust | Status | Notes |
|--------|------|--------|-------|
| `relative_to(other, *, walk_up=False)` | `relative_to(py, other)` | ⚠️ | **Rust manque le paramètre `walk_up`** |
| `is_relative_to(other)` | `is_relative_to(other)` | ✅ | Identique |
| `is_absolute()` | `is_absolute()` | ✅ | Identique |
| `is_reserved()` | ❌ **MANQUANT** | ❌ | Vérifie les noms réservés Windows (deprecated en Python 3.13+) |

### Méthodes de matching/globbing

| Python | Rust | Status | Notes |
|--------|------|--------|-------|
| `match(path_pattern, *, case_sensitive=None)` | ❌ **MANQUANT** | ❌ | Match de la droite vers la gauche |
| `full_match(pattern, *, case_sensitive=None)` | `full_match(pattern)` | ⚠️ | **Rust manque `case_sensitive` param** |
| (helper) `_glob_match(pattern)` | `_glob_match(pattern)` | ✅ | Implémentation custom Rust |
| (helper) `_match_recursive(...)` | `_match_recursive(...)` | ✅ | Implémentation custom Rust |
| (helper) `_segment_matches(...)` | `_segment_matches(...)` | ✅ | Implémentation custom Rust |

### Opérateurs

| Python | Rust | Status | Notes |
|--------|------|--------|-------|
| `__truediv__(key)` | `__truediv__(py, key)` | ✅ | `/` operator |
| `__rtruediv__(key)` | `__rtruediv__(py, key)` | ✅ | Reverse `/` |
| `__eq__(other)` | `__eq__(other)` | ⚠️ | Python vérifie aussi `parser`, Rust NON |
| `__hash__()` | `__hash__()` | ⚠️ | Python cache le hash, Rust NON |
| `__lt__(other)` | `__lt__(other)` | ⚠️ | Python vérifie `parser`, Rust NON |
| `__le__(other)` | `__le__(other)` | ⚠️ | Idem |
| `__gt__(other)` | `__gt__(other)` | ⚠️ | Idem |
| `__ge__(other)` | `__ge__(other)` | ⚠️ | Idem |
| `__str__()` | `__str__()` | ✅ | Identique |
| `__repr__()` | `__repr__()` | ✅ | Identique |
| `__fspath__()` | `__fspath__()` | ✅ | Identique |
| `__reduce__()` | ❌ **MANQUANT** | ❌ | Pour pickle support |

---

## 🚨 Incohérences Critiques

### 1. **Méthodes manquantes en Rust**

#### Haute priorité (impact fonctionnel)

- ❌ `_from_parsed_parts(drv, root, tail)` - **Critique**: utilisé partout en Python
- ❌ `_format_parsed_parts(drv, root, tail)` - **Critique**: reconstruction de string
- ❌ `_parse_pattern(pattern)` - Nécessaire pour glob avancé
- ❌ `match(path_pattern, *, case_sensitive=None)` - Fonctionnalité publique importante
- ❌ `is_reserved()` - Vérifie noms Windows réservés (CON, PRN, etc.)
- ❌ `__reduce__()` - Support pickle/serialization

#### Priorité moyenne (accesseurs/propriétés)

- ❌ `_raw_path` property - Joint `_raw_paths` en un string
- ❌ `_tail` property - Accès direct aux parts

### 2. **Paramètres manquants en Rust**

| Méthode | Paramètre manquant | Impact |
|---------|-------------------|--------|
| `relative_to()` | `walk_up: bool` | Ne peut pas utiliser `..` pour remonter |
| `full_match()` | `case_sensitive: bool \| None` | Toujours case-sensitive sur Posix, insensitive sur Windows |

### 3. **Différences de comportement**

#### `__eq__()` et comparaisons

```python
# Python
def __eq__(self, other):
    if not isinstance(other, PurePath):
        return NotImplemented
    return self._str_normcase == other._str_normcase and self.parser is other.parser
    #                                                   ^^^^^^^^^^^^^^^^^^^^^^^^
    #                                                   Vérifie que c'est le même système!
```

```rust
// Rust
fn __eq__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
    match other.extract::<Py<$class_name>>() {
        Ok(other_py) => Python::attach(|py| {
            Ok(self.str_normcase() == other_py.borrow(py).str_normcase())
            // ❌ Ne vérifie PAS le type de séparateur!
        }),
        Err(_) => Ok(false),
    }
}
```

**Problème**: En Rust, `PurePosixPath("/foo") == PureWindowsPath("/foo")` retourne `true`, alors qu'en Python c'est `false`.

#### Comparaisons (`__lt__`, `__le__`, etc.)

Même problème: Python vérifie `self.parser is other.parser`, Rust NON.

#### Hash caching

```python
# Python - cache le hash
def __hash__(self):
    try:
        return self._hash
    except AttributeError:
        self._hash = hash(self._str_normcase)
        return self._hash
```

```rust
// Rust - recalcule à chaque fois
fn __hash__(&self) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    self.str_normcase().hash(&mut hasher);
    hasher.finish()
}
```

**Impact**: Performance - si un objet est hashé plusieurs fois, Rust fera le travail plusieurs fois.

---

## 📝 Méthodes en trop / spécifiques Rust

| Méthode Rust | Équivalent Python | Note |
|--------------|-------------------|------|
| `str_repr()` | `__str__()` | Méthode privée interne |
| `str_repr_original()` | ❌ N'existe pas | Stocke la version non-normalisée |
| `parsed_parts()` | Accès à `_drv`, `_root`, `_tail` séparément | Retourne toute la structure |
| `str_normcase()` | `_str_normcase` property | En Rust c'est une méthode |
| `parts_normcase()` | `_parts_normcase` property | Idem |
| `extract_path_strs()` | Intégré dans `__init__` | Helper Rust pour conversion |
| `from_str_repr()` | `_from_parsed_string()` | Similaire mais moins flexible |
| `compute_str_repr()` | Intégré dans `__str__` | Helper Rust |

---

## 🏗️ Différences d'architecture

### Python: Parsing lazy par composant

```python
@property
def drive(self):
    try:
        return self._drv
    except AttributeError:
        # Parse TOUT mais on peut accéder juste à drive
        self._drv, self._root, self._tail_cached = self._parse_path(self._raw_path)
        return self._drv
```

### Rust: Parsing tout-en-un

```rust
fn parsed_parts(&self) -> &ParsedParts {
    self.parsed.get_or_init(|| <$separator>::parse(self.str_repr()))
    // Parse TOUT d'un coup, retourne la struct complète
}
```

**Avantage Rust**: Un seul parsing
**Inconvénient Rust**: Si on veut juste `drive`, on parse quand même `root` et `parts`

---

## 🎯 Recommandations

### 1. Ajouter les méthodes manquantes critiques

```rust
// À ajouter dans macros.rs
impl $class_name {
    // Équivalent de _from_parsed_parts
    fn from_parsed_parts(py: Python, drv: String, root: String, tail: Vec<String>) -> PyResult<Py<Self>> {
        let parsed = ParsedParts {
            drive: drv,
            root: root,
            parts: tail,
        };
        
        // Construire le string à partir des parties
        let str_repr = <$separator>::format_parsed_parts(&parsed);
        
        let path = Self::from_str_repr(str_repr);
        let _ = path.parsed.set(parsed); // Réutiliser les parties parsées!
        Py::new(py, path)
    }
    
    // Équivalent de _raw_path
    fn raw_path(&self) -> String {
        if self._raw_path_tuple.len() == 1 {
            return self._raw_path_tuple[0].clone();
        }
        if !self._raw_path_tuple.is_empty() {
            Python::attach(|py| {
                PyModule::import(py, <$separator>::MODULE_NAME)
                    .and_then(|m| m.getattr("join"))
                    .and_then(|f| f.call1(PyTuple::new(py, &self._raw_path_tuple)?))
                    .and_then(|r| r.extract())
                    .unwrap_or_default()
            })
        } else {
            String::new()
        }
    }
}

#[pymethods]
impl $class_name {
    // Support pickle
    fn __reduce__(&self, py: Python) -> PyResult<(PyObject, Py<PyTuple>)> {
        let cls = self.into_py(py).getattr(py, "__class__")?;
        let args = PyTuple::new(py, &self._raw_path_tuple)?;
        Ok((cls, args.into()))
    }
    
    // match() manquant
    fn match_(&self, path_pattern: &str, case_sensitive: Option<bool>) -> PyResult<bool> {
        // Implémentation
        todo!()
    }
    
    // is_reserved() manquant
    fn is_reserved(&self) -> bool {
        // Sur Windows, vérifier CON, PRN, AUX, NUL, COM1-9, LPT1-9
        if <$separator>::MODULE_NAME == "ntpath" {
            // Implémentation Windows
            todo!()
        } else {
            false
        }
    }
}
```

### 2. Ajouter à separators.rs

```rust
impl PosixSeparator {
    // Équivalent de _format_parsed_parts
    pub fn format_parsed_parts(parsed: &ParsedParts) -> String {
        if !parsed.drive.is_empty() || !parsed.root.is_empty() {
            format!("{}{}{}", 
                parsed.drive, 
                parsed.root, 
                parsed.parts.join(&Self::SEP.to_string())
            )
        } else if !parsed.parts.is_empty() && parsed.parts[0].contains(':') {
            // Si premier element a un drive, ajouter "."
            format!(".{}{}", Self::SEP, parsed.parts.join(&Self::SEP.to_string()))
        } else {
            parsed.parts.join(&Self::SEP.to_string())
        }
    }
}

impl WindowsSeparator {
    pub fn format_parsed_parts(parsed: &ParsedParts) -> String {
        // Idem mais avec "\\"
        todo!()
    }
}
```

### 3. Fixer les comparaisons

```rust
// Modifier __eq__ pour vérifier le type
fn __eq__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
    // Vérifier que c'est bien le même type (PurePosixPath vs PureWindowsPath)
    if !other.is_instance_of::<$class_name>() {
        return Ok(false);
    }
    
    match other.extract::<Py<$class_name>>() {
        Ok(other_py) => Python::attach(|py| {
            Ok(self.str_normcase() == other_py.borrow(py).str_normcase())
        }),
        Err(_) => Ok(false),
    }
}
```

### 4. Ajouter le cache de hash

```rust
// Dans la struct
pub struct $class_name {
    // ... champs existants ...
    _hash_cached: OnceLock<u64>,  // ← AJOUTER
}

// Dans __hash__
fn __hash__(&self) -> u64 {
    *self._hash_cached.get_or_init(|| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.str_normcase().hash(&mut hasher);
        hasher.finish()
    })
}
```

### 5. Ajouter les paramètres manquants

```rust
// relative_to avec walk_up
#[pyo3(signature = (other, *, walk_up=false))]
fn relative_to(&self, py: Python, other: &Bound<PyAny>, walk_up: bool) -> PyResult<Py<Self>> {
    // Implémentation avec support de walk_up
    todo!()
}

// full_match avec case_sensitive
#[pyo3(signature = (pattern, *, case_sensitive=None))]
fn full_match(&self, pattern: &str, case_sensitive: Option<bool>) -> PyResult<bool> {
    let case_sensitive = case_sensitive.unwrap_or_else(|| {
        <$separator>::MODULE_NAME == "posixpath"
    });
    // Implémentation
    todo!()
}
```

---

## 📋 Checklist de mise en conformité

### Structure et champs

- [ ] Ajouter `_hash_cached: OnceLock<u64>`

### Méthodes privées/helpers manquantes

- [ ] `_from_parsed_parts(drv, root, tail)`
- [ ] `_format_parsed_parts(drv, root, tail)` en `separators.rs`
- [ ] `_parse_pattern(pattern)`
- [ ] `_raw_path` property ou méthode

### Méthodes publiques manquantes

- [ ] `match(path_pattern, *, case_sensitive=None)`
- [ ] `is_reserved()`
- [ ] `__reduce__()` pour pickle

### Paramètres manquants

- [ ] `relative_to(..., walk_up: bool)`
- [ ] `full_match(..., case_sensitive: Option<bool>)`

### Corrections de comportement

- [ ] `__eq__()` - vérifier type de path (Posix vs Windows)
- [ ] `__lt__(), __le__(), __gt__(), __ge__()` - idem
- [ ] `__hash__()` - cacher le résultat
- [ ] `as_uri()` - ajouter deprecation warnings?

### Tests à ajouter

- [ ] Test `PurePosixPath != PureWindowsPath` même string
- [ ] Test `relative_to()` avec `walk_up=True`
- [ ] Test `full_match()` avec `case_sensitive`
- [ ] Test `match()` (méthode complète)
- [ ] Test `is_reserved()` sur Windows
- [ ] Test pickle/unpickle

---

## 💡 Clarifications sur les confusions

### `_from_parsed_parts` vs `ParsedParts`

**Q**: "_from_parsed_parts n'existe pas en Rust, du coup je suis confu, mais après je vois qu'on a une struct ParsedParts"

**R**:

- **`ParsedParts`** (struct Rust) = conteneur pour stocker `drive`, `root`, `parts` ensemble
  - Équivalent des 3 champs Python: `_drv`, `_root`, `_tail_cached`
  - Créé par `<Separator>::parse()`

- **`_from_parsed_parts()`** (méthode Python) = constructeur qui crée un nouveau `PurePath` à partir de parties déjà parsées
  - Évite de re-parser le string
  - Réutilise les parties connues
  - **N'existe PAS en Rust actuellement**

**Ce qui manque en Rust**: Une méthode qui prend `ParsedParts` et retourne un nouveau `Path` sans repasser par le string.

Actuellement Rust fait:

```rust
// ❌ Inefficace
ParsedParts → String (via format) → parse() → ParsedParts
```

Ce qu'il faudrait:

```rust
// ✅ Optimal
ParsedParts → nouveau Path (réutilise les parts directement)
```

---

## 🔬 Impact Performance

| Opération | Python | Rust actuel | Potentiel Rust optimisé |
|-----------|--------|-------------|------------------------|
| `path.parent` | Parse 1x, réutilise parties | Parse 1x, reconstruit string, re-parse | Parse 1x, réutilise parties |
| `path.with_name()` | Parse 1x, réutilise parties | Parse 1x, reconstruit string, re-parse | Parse 1x, réutilise parties |
| `path1 == path2` | Hash cachés | Hash recalculés | Hash cachés |
| Multiple `__hash__()` | 1 calcul | N calculs | 1 calcul |

**Conclusion**: Rust peut être plus rapide que Python grâce au parsing unique de `ParsedParts`, MAIS il perd cet avantage en reconstituant inutilement des strings. Avec les corrections, Rust devrait être significativement plus rapide.
