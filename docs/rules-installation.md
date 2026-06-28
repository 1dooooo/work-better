# Rules

## Structure

Rules are organized into a **common** layer plus **language-specific** directories:

```
rules/
├── common/          # Language-agnostic principles (always install)
│   ├── coding-style.md
│   ├── git-workflow.md
│   ├── testing.md
│   ├── performance.md
│   ├── patterns.md
│   ├── hooks.md
│   ├── agents.md
│   └── security.md
├── frontend/        # Frontend-specific (TypeScript, React, CSS, HTML)
│   ├── coding-style.md
│   ├── testing.md
│   ├── performance.md
│   ├── patterns.md
│   ├── hooks.md
│   ├── design-quality.md
│   └── security.md
├── rust/            # Rust-specific
│   ├── coding-style.md
│   ├── testing.md
│   ├── patterns.md
│   ├── hooks.md
│   └── security.md
└── workflow.md      # Multi-Agent workflow rules
```

- **common/** contains universal principles — no language-specific code examples.
- **frontend/** merges TypeScript and web-specific rules for frontend development.
- **rust/** contains Rust-specific patterns, testing, and security rules.
- **workflow.md** defines the mandatory multi-agent workflow for code changes.

## Installation

### Option 1: Install Script (Recommended)

```bash
# Install common + one or more language-specific rule sets
./install.sh frontend
./install.sh rust
./install.sh typescript python
```

### Option 2: Manual Installation

> **Important:** Copy entire directories — do NOT flatten with `/*`.
> Common and language-specific directories contain files with the same names.
> Flattening them into one directory causes language-specific files to overwrite
> common rules, and breaks the relative `../common/` references used by
> language-specific files.
>
> Use the ECC-owned namespace below for user-level Claude installs. Flat
> package-level destinations can collide with non-ECC rule packs and do not
> match the main README guidance.

```bash
# Create the ECC rule namespace once.
mkdir -p ~/.claude/rules/ecc

# Install common rules (required for all projects)
cp -r rules/common ~/.claude/rules/ecc/

# Install language-specific rules based on your project's tech stack
cp -r rules/frontend ~/.claude/rules/ecc/
cp -r rules/rust ~/.claude/rules/ecc/

# Attention ! ! ! Configure according to your actual project requirements; the configuration here is for reference only.
```

For project-local rules, use the same namespace under the project root:

```bash
mkdir -p .claude/rules/ecc
cp -r rules/common .claude/rules/ecc/
cp -r rules/frontend .claude/rules/ecc/
cp -r rules/rust .claude/rules/ecc/
```

## Rules vs Skills

- **Rules** define standards, conventions, and checklists that apply broadly (e.g., "80% test coverage", "no hardcoded secrets").
- **Skills** (`skills/` directory) provide deep, actionable reference material for specific tasks (e.g., `python-patterns`, `golang-testing`).

Language-specific rule files reference relevant skills where appropriate. Rules tell you *what* to do; skills tell you *how* to do it.

## Adding a New Language

To add support for a new language (e.g., `golang/`):

1. Create a `rules/golang/` directory
2. Add files that extend the common rules:
   - `coding-style.md` — formatting tools, idioms, error handling patterns
   - `testing.md` — test framework, coverage tools, test organization
   - `patterns.md` — language-specific design patterns
   - `hooks.md` — PostToolUse hooks for formatters, linters, type checkers
   - `security.md` — secret management, security scanning tools
3. Each file should start with:
   ```
   > This file extends [common/xxx.md](../common/xxx.md) with <Language> specific content.
   ```
4. Reference existing skills if available, or create new ones under `skills/`.

For non-language domains like `frontend/`, follow the same layered pattern when there is enough reusable domain-specific guidance to justify a standalone ruleset.

## Rule Priority

When language-specific rules and common rules conflict, **language-specific rules take precedence** (specific overrides general). This follows the standard layered configuration pattern (similar to CSS specificity or `.gitignore` precedence).

- `rules/common/` defines universal defaults applicable to all projects.
- `rules/frontend/`, `rules/rust/`, etc. override those defaults where language idioms differ.

### Example

`common/coding-style.md` recommends immutability as a default principle. A language-specific `golang/coding-style.md` can override this:

> Idiomatic Go uses pointer receivers for struct mutation — see [common/coding-style.md](../common/coding-style.md) for the general principle, but Go-idiomatic mutation is preferred here.

### Common rules with override notes

Rules in `rules/common/` that may be overridden by language-specific files are marked with:

> **Language note**: This rule may be overridden by language-specific rules for languages where this pattern is not idiomatic.
