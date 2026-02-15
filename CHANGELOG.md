# Changelog

All notable changes to Quantum Minefield will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-02-16

### Added

- **Rust core library (`qmf-core`)** — pure game logic with no runtime dependencies.
  - `QuantumGrid` — minefield state machine with deferred mine placement (first click is always safe).
  - `Circuit` — quantum-gate pipeline (Hadamard, Phase Shift, Not) that scrambles probability hints per difficulty.
  - `Entanglement` — paired cells whose probabilities are correlated on observation.
  - `SplitMix64` — hand-rolled PRNG for deterministic, reproducible games (no `rand` crate dependency).
- **Wasm bridge (`qmf-wasm`)** — `wasm-bindgen` bindings exposing `init_game`, `reveal_cell`, `contain_cell`, snapshot, and probability cloud APIs.
- **Next.js 16 frontend (`@qmf/web`)** — App Router + Turbopack single-page game UI.
  - Lobby with difficulty selection (Observer / Researcher / Theorist) and instructions modal.
  - `QuantumBoard` — glassmorphism game board with entropy progress bar, containment charges HUD, and quantum inspector toggle.
  - `useQuantumGame` hook — React↔Wasm bridge with lifecycle management and error recovery.
- **Wavefunction Purification win condition** — player wins by driving system entropy to zero (resolving every cell via reveal or containment), replacing the classical "flag all mines" approach.
- **Containment mechanic** — right-click to lock down a suspected mine; charges are limited to the exact mine count and incorrect containment wastes a charge.
- **Difficulty-scaled quantum circuits:**
  - _Observer_ (8×8, 10 mines) — mild Phase Shift, low entanglement.
  - _Researcher_ (12×12, 24 mines) — Hadamard + Phase Shift, moderate entanglement.
  - _Theorist_ (16×16, 50 mines) — double Hadamard + Phase Shift, high entanglement.
- **Flood fill** — stack-based iterative DFS auto-reveals adjacent safe cells with zero neighboring mines.
- **Seeded PRNG** — each game gets a unique seed; `init_game_seeded` enables deterministic replays.
- **Build pipeline** — `wasm-pack` build scripts (`pnpm wasm:dev` / `pnpm wasm:build`), pre-built wasm artifacts for zero-Rust Vercel deploys.
- **21 Rust unit tests** covering grid mechanics, circuit math, RNG properties, and win/loss conditions.
