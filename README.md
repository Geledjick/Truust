# Truust 🌲
A blazing-fast `tree` alternative written in Rust, with a powerful pipeline CLI.

## Installation
```bash
cargo install --path .
```

## Usage
If `path` is omitted, the current directory is used.

Commands are chained into a **pipeline** — each one transforms or reads the current state:
```bash
truust find “*.rs” remove “target” tree
```
│ │ │
│ │ └─ display as tree
│ └─ remove target/ from results
└─ filter to .rs files

## Commands
| Command | Type | Description |
|---------|------|-------------|
| `find <pattern>` | Filter | Keep only nodes matching glob pattern |
| `remove <pattern>` | Filter | Remove nodes matching glob pattern |
| `tree` | Output | Display results as a tree |
| `print` | Output | List results flat |
| `amount` | Output | Count files, dirs and symlinks |

## Examples
```bash
# Show the current directory as a tree
truust tree

# Show a specific path
truust ~/projects/myapp tree

# Find all Rust source files
truust find "*.rs" print

# Find all Rust files, show as tree
truust find "*.rs" tree

# Ignore common noise, show tree
truust remove "target" remove ".git" remove "node_modules" tree

# Find files, count them
truust find "*.cpp" amount

# Chain everything
truust remove "target" find "*.rs" tree amount

# Glob patterns — all supported
truust find "*.{rs,toml}" print      # multiple extensions
truust find "main*" print            # starts with
truust find "src/*.rs" print         # path match
truust find "**/test*.rs" tree       # recursive
```

## Glob Patterns
Truust uses [`globset`](https://crates.io/crates/globset) for pattern matching.

| Pattern | Matches |
|---------|---------|
| `*.rs` | Any `.rs` file |
| `*.{rs,toml}` | `.rs` or `.toml` files |
| `main*` | Anything starting with `main` |
| `src/*.rs` | `.rs` files inside `src/` |
| `**/tests/*.rs` | `.rs` files in any `tests/` dir |

> Patterns with `/` match against the full path.
> Patterns without `/` match against the filename only.

## Pipeline Design
Unlike traditional `tree`, Truust is built around a **command pipeline**:

- **Filter commands** (`find`, `remove`) narrow the working set
- **Output commands** (`tree`, `print`, `amount`) read and display it
- Commands can be **chained freely** in any order
- If no output command is given, `print` is added automatically