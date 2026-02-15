"use client";

import { useState } from "react";

export interface GameConfig {
  width: number;
  height: number;
  mineCount: number;
  label: string;
}

const DIFFICULTIES: GameConfig[] = [
  { width: 8, height: 8, mineCount: 10, label: "Observer" },
  { width: 12, height: 12, mineCount: 24, label: "Researcher" },
  { width: 16, height: 16, mineCount: 50, label: "Theorist" },
];

interface LobbyProps {
  readonly onStartGame: (config: GameConfig) => void;
}

export function Lobby({ onStartGame }: LobbyProps) {
  const [showInstructions, setShowInstructions] = useState(false);
  const [selectedDifficulty, setSelectedDifficulty] = useState(1);

  const selected = DIFFICULTIES[selectedDifficulty];

  return (
    <div className="lobby-container">
      <div className="lobby-backdrop" />

      <main className="lobby-content">
        <div className="lobby-hero">
          <div className="hero-icon">⚛</div>
          <h1 className="hero-title">Quantum Minefield</h1>
          <p className="hero-subtitle">The Schrödinger&apos;s Logic Game</p>
        </div>

        <section className="glass lobby-card">
          <h2 className="card-heading">Select Difficulty</h2>

          <div className="difficulty-grid">
            {DIFFICULTIES.map((diff, i) => (
              <button
                key={diff.label}
                type="button"
                className={`difficulty-option ${i === selectedDifficulty ? "selected" : ""}`}
                onClick={() => setSelectedDifficulty(i)}
              >
                <span className="diff-label">{diff.label}</span>
                <span className="diff-meta">
                  {diff.width}×{diff.height} &middot; {diff.mineCount} mines
                </span>
                <span className="diff-description">
                  {i === 0
                    ? "Gentle introduction to quantum uncertainty"
                    : i === 1
                      ? "Standard challenge with entangled pairs"
                      : "Dense probability fields for experts"}
                </span>
              </button>
            ))}
          </div>

          <button
            type="button"
            className="btn-primary"
            onClick={() => onStartGame(selected)}
          >
            New Game
          </button>

          <button
            type="button"
            className="btn-secondary"
            onClick={() => setShowInstructions(true)}
          >
            How to Play
          </button>
        </section>

        <footer className="lobby-footer">
          <span>Powered by Rust + WebAssembly</span>
        </footer>
      </main>

      {showInstructions && (
        <Instructions onClose={() => setShowInstructions(false)} />
      )}
    </div>
  );
}

function Instructions({ onClose }: { readonly onClose: () => void }) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="glass modal-content" onClick={(e) => e.stopPropagation()}>
        <header className="modal-header">
          <h2>How to Play</h2>
          <button type="button" className="modal-close" onClick={onClose}>
            ✕
          </button>
        </header>

        <div className="instructions-body">
          <div className="instruction-section">
            <div className="instruction-icon">🎯</div>
            <div>
              <h3>Objective — Wavefunction Purification</h3>
              <p>
                Drive the system&apos;s entropy to zero by resolving{" "}
                <strong>every cell</strong>: reveal safe cells (left-click) and
                contain mines (right-click). When no cell remains in
                superposition, the minefield is <strong>purified</strong> and
                you win.
              </p>
            </div>
          </div>

          <div className="instruction-section">
            <div className="instruction-icon">👆</div>
            <div>
              <h3>Reveal (Left-click)</h3>
              <p>
                Click a cell to <strong>collapse its wavefunction</strong>. If
                the cell is safe, it shows the number of adjacent mines.
                Revealing a cell with zero adjacent mines cascades outward
                automatically. If the cell is a mine — detonation, game over.
              </p>
            </div>
          </div>

          <div className="instruction-section">
            <div className="instruction-icon">🛡</div>
            <div>
              <h3>Containment (Right-click)</h3>
              <p>
                Right-click to <strong>contain</strong> a cell you suspect is a
                mine. You have exactly as many containment charges as there are
                mines. A correct containment locks the mine safely. A{" "}
                <strong>wrong containment</strong> reveals the cell as safe but{" "}
                <strong>wastes a charge</strong> — run out and you can&apos;t
                resolve the remaining mines.
              </p>
            </div>
          </div>

          <div className="instruction-section">
            <div className="instruction-icon">🔗</div>
            <div>
              <h3>Entanglement</h3>
              <p>
                Some cells are <strong>quantum-entangled</strong>. Resolving one
                cell shifts the displayed probability of its entangled partner —
                containing a mine makes its partner appear safer, and vice
                versa.
              </p>
            </div>
          </div>

          <div className="instruction-section">
            <div className="instruction-icon">🔬</div>
            <div>
              <h3>Quantum Inspector</h3>
              <p>
                Toggle the inspector to see probability hints in Dirac notation
                (<code>|0⟩</code> = safe, <code>|1⟩</code> = mine). Higher
                difficulty scrambles these hints more via quantum circuit gates.
              </p>
            </div>
          </div>

          <div className="instruction-section">
            <div className="instruction-icon">⚡</div>
            <div>
              <h3>Quantum Circuits</h3>
              <p>
                Quantum gates (Hadamard, Phase Shift) scramble the displayed
                probabilities. At Observer difficulty, hints are reliable. At
                Theorist difficulty, the circuit heavily distorts them — making
                deduction harder.
              </p>
            </div>
          </div>
        </div>

        <button type="button" className="btn-primary" onClick={onClose}>
          Got It
        </button>
      </div>
    </div>
  );
}
