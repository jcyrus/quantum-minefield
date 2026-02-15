# Contributing to Quantum Minefield

Thanks for your interest in contributing! This guide will get you set up.

## Prerequisites

| Tool      | Version                            | Install                                                                      |
| --------- | ---------------------------------- | ---------------------------------------------------------------------------- |
| Rust      | stable (via `rust-toolchain.toml`) | [rustup.rs](https://rustup.rs)                                               |
| wasm-pack | latest                             | `cargo install wasm-pack`                                                    |
| Node.js   | 24 LTS (see `.nvmrc`)              | [nvm](https://github.com/nvm-sh/nvm) or [fnm](https://github.com/Schniz/fnm) |
| pnpm      | 10.x                               | `corepack enable` or `npm i -g pnpm`                                         |

The `wasm32-unknown-unknown` target is installed automatically by `rust-toolchain.toml`.

## Getting Started

```bash
# Clone the repo
git clone https://github.com/jcyrus/quantum-minefield.git
cd quantum-minefield

# Install JS dependencies
pnpm install

# Build wasm + start dev server
pnpm dev
```

This runs `wasm-pack build` (dev mode) then starts the Next.js dev server at `http://localhost:3000`.

## Project Structure

```
crates/
  qmf-core/   — Pure Rust game logic (no wasm deps)
  qmf-wasm/   — wasm-bindgen bridge
apps/
  web/         — Next.js 16 frontend
scripts/       — Build helpers
```

## Development Workflow

### Rust changes

1. Edit files in `crates/`.
2. Run tests: `cargo test --workspace`.
3. Rebuild wasm: `pnpm wasm:dev`.
4. Hard-refresh the browser (Cmd+Shift+R / Ctrl+Shift+R).

### Frontend changes

1. Edit files in `apps/web/src/`.
2. Turbopack hot-reloads automatically — no restart needed.
3. Type-check: `cd apps/web && pnpm typecheck`.

### Running Tests

```bash
# Rust unit tests (21 tests)
cargo test --workspace

# TypeScript type checking
cd apps/web && pnpm typecheck
```

## Commit Conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

feat(core):   new game mechanic or Rust logic
fix(wasm):    bug fix in the wasm bridge
feat(web):    new UI component or hook
fix(web):     frontend bug fix
docs:         documentation only
chore:        build scripts, deps, tooling
test(core):   new or updated Rust tests
refactor:     code restructuring without behavior change
```

## Pull Request Process

1. **Fork & branch** — create a feature branch from `main`.
2. **Keep PRs focused** — one logical change per PR.
3. **Tests pass** — `cargo test --workspace` must be green.
4. **Types pass** — `pnpm typecheck` in `apps/web/` must be clean.
5. **Describe your change** — explain _what_ and _why_ in the PR body.

## Areas for Contribution

Here are some good places to start:

- **Mobile support** — long-press gesture for containment (right-click doesn't work on touch).
- **Accessibility** — ARIA labels, keyboard navigation, screen reader support.
- **Replay system** — share seeds via URL params for deterministic game replay.
- **Sound effects** — audio feedback for reveal, containment, detonation.
- **Leaderboard** — track fastest purification times per difficulty.
- **Additional tests** — edge cases, integration tests, browser E2E tests.
- **Documentation** — improve inline code docs, add architecture diagrams.

## Code Style

- **Rust:** standard `rustfmt` formatting, `clippy` clean.
- **TypeScript:** project-configured settings (strict mode, no unused variables).
- **CSS:** BEM-ish class naming, CSS custom properties for theming.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
