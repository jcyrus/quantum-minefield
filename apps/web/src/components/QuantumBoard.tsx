"use client";

import { useCallback } from "react";
import { useQuantumGame } from "@/hooks/useQuantumGame";

interface QuantumBoardProps {
  readonly width: number;
  readonly height: number;
  readonly mineCount: number;
  readonly difficultyLabel: string;
  readonly onBackToLobby: () => void;
}

function cellClassName(state: string, inspectorEnabled: boolean): string {
  const base = "cell";
  if (state === "revealed") return `${base} cell-revealed`;
  if (state === "detonated") return `${base} cell-detonated`;
  if (state === "contained") return `${base} cell-contained`;
  if (state === "superposition" && inspectorEnabled)
    return `${base} cell-inspector`;
  return base;
}

function renderCellContent(
  inspectorEnabled: boolean,
  cell: {
    state: {
      state: string;
      probability?: number;
      adjacent_mines?: number;
    };
  },
) {
  const { state } = cell;

  if (state.state === "revealed") {
    const count = state.adjacent_mines ?? 0;
    if (count === 0) return <span className="cell-empty" />;
    return <span className={`cell-count cell-count-${count}`}>{count}</span>;
  }

  if (state.state === "contained") {
    return <span className="cell-contained-icon">🛡</span>;
  }

  if (state.state === "detonated") {
    return <span className="cell-mine-icon">☢</span>;
  }

  // Superposition
  if (inspectorEnabled) {
    const p = state.probability ?? 0;
    return (
      <span className="cell-quantum-label">
        <span className="ket-safe">|0⟩</span>
        <span className="ket-value">{(1 - p).toFixed(2)}</span>
        <span className="ket-mine">|1⟩</span>
        <span className="ket-value">{p.toFixed(2)}</span>
      </span>
    );
  }

  return <span className="cell-hidden">?</span>;
}

export function QuantumBoard({
  width,
  height,
  mineCount,
  difficultyLabel,
  onBackToLobby,
}: QuantumBoardProps) {
  const {
    ready,
    error,
    inspectorEnabled,
    grid,
    probabilityCloud,
    revealCell,
    containCell,
    toggleInspector,
    newGame,
  } = useQuantumGame(width, height, mineCount, difficultyLabel.toLowerCase());

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, x: number, y: number) => {
      e.preventDefault();
      containCell(x, y);
    },
    [containCell],
  );

  if (error) {
    return (
      <div className="game-container">
        <div className="glass shell error-state">
          <h2>⚠️ Initialization Failed</h2>
          <p>{error}</p>
          <p className="hint">
            Run <code>pnpm wasm:dev</code> to build the Wasm module first.
          </p>
          <button
            type="button"
            className="btn-secondary"
            onClick={onBackToLobby}
          >
            Back to Lobby
          </button>
        </div>
      </div>
    );
  }

  if (!ready || !grid) {
    return (
      <div className="game-container">
        <div className="glass shell loading-state">
          <div className="spinner" />
          <p>Initializing quantum substrate…</p>
        </div>
      </div>
    );
  }

  const isFinished = grid.game_over || grid.won;
  const entropyPct = (1 - grid.entropy) * 100;
  const chargesLow =
    !isFinished &&
    grid.containment_charges > 0 &&
    grid.containment_charges <= Math.ceil(mineCount * 0.3);

  return (
    <div className="game-container">
      {/* Top toolbar */}
      <nav className="glass game-toolbar">
        <button type="button" className="btn-ghost" onClick={onBackToLobby}>
          ← Lobby
        </button>

        <div className="toolbar-center">
          <span className="toolbar-title">Quantum Minefield</span>
          <span className="toolbar-difficulty">{difficultyLabel}</span>
        </div>

        <div className="toolbar-actions">
          <button
            type="button"
            className={`btn-ghost ${inspectorEnabled ? "btn-active" : ""}`}
            onClick={toggleInspector}
            title="Toggle Quantum Inspector"
          >
            🔬
          </button>
          <button
            type="button"
            className="btn-ghost"
            onClick={newGame}
            title="New Game"
          >
            🔄
          </button>
        </div>
      </nav>

      {/* Stats HUD */}
      <div className="glass stats-hud">
        <div className="stat">
          <span className={`stat-value ${chargesLow ? "stat-warning" : ""}`}>
            ⬡ {grid.containment_charges}
          </span>
          <span className="stat-label">Charges</span>
        </div>
        <div className="stat stat-entropy">
          <div className="entropy-track">
            <div
              className="entropy-fill"
              style={{ width: `${entropyPct.toFixed(1)}%` }}
            />
          </div>
          <span className="stat-label">
            System Purity {entropyPct.toFixed(0)}%
          </span>
        </div>
        <div className="stat">
          <span className="stat-value">{mineCount}</span>
          <span className="stat-label">Mines</span>
        </div>
        <div className="stat">
          <span
            className={`stat-value stat-status ${isFinished ? (grid.won ? "stat-won" : "stat-lost") : "stat-active"}`}
          >
            {grid.game_over ? "DETONATED" : grid.won ? "PURIFIED" : "COHERENT"}
          </span>
          <span className="stat-label">Field</span>
        </div>
      </div>

      {/* Board */}
      <section className="glass board-shell">
        <div
          className="board-grid"
          style={{
            gridTemplateColumns: `repeat(${grid.width}, minmax(0, 1fr))`,
          }}
        >
          {grid.cells.map((cell, index) => (
            <button
              key={`${cell.x}-${cell.y}-${index}`}
              type="button"
              className={cellClassName(cell.state.state, inspectorEnabled)}
              onClick={() => revealCell(cell.x, cell.y)}
              onContextMenu={(e) => handleContextMenu(e, cell.x, cell.y)}
              disabled={isFinished}
              title={`(${cell.x}, ${cell.y}) — Left: reveal · Right: contain`}
            >
              {renderCellContent(inspectorEnabled, cell)}
            </button>
          ))}
        </div>
      </section>

      {/* Game result card */}
      {isFinished && (
        <div
          className={`glass game-result-card ${grid.game_over ? "result-lost" : "result-won"}`}
        >
          <div className="result-header">
            <span className="result-icon">{grid.game_over ? "☢️" : "✨"}</span>
            <div>
              <strong>
                {grid.game_over ? "Detonation" : "System Purified"}
              </strong>
              <p>
                {grid.game_over
                  ? "A mine collapsed into an unsafe eigenstate. The wavefunction is irreversibly disturbed."
                  : "All wavefunctions resolved. Entropy: 0. The minefield has been fully purified."}
              </p>
            </div>
          </div>
          <div className="result-actions">
            <button type="button" className="btn-primary" onClick={newGame}>
              Play Again
            </button>
            <button
              type="button"
              className="btn-secondary"
              onClick={onBackToLobby}
            >
              Change Difficulty
            </button>
          </div>
        </div>
      )}

      {/* Inspector panel */}
      {inspectorEnabled && probabilityCloud.length > 0 && (
        <section className="glass inspector-panel">
          <div className="inspector-header">
            <span>🔬 Quantum Inspector</span>
            <span className="inspector-hint">
              Probability amplitudes for first 24 cells
            </span>
          </div>
          <div className="inspector-grid">
            {probabilityCloud.slice(0, 24).map((p, i) => (
              <div key={`prob-${i}`} className="inspector-item">
                <span className="inspector-cell-id">
                  ψ<sub>{i}</sub>
                </span>
                <div className="inspector-bar-track">
                  <div
                    className="inspector-bar-fill"
                    style={{ width: `${(p * 100).toFixed(0)}%` }}
                  />
                </div>
                <span className="inspector-ket">
                  |0⟩ {(1 - p).toFixed(2)} + |1⟩ {p.toFixed(2)}
                </span>
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
