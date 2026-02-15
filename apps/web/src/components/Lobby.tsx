"use client";

import Link from "next/link";
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

          <Link href="/how-to-play" className="btn-secondary">
            Cadet Briefing (How to Play)
          </Link>
        </section>

        <footer className="lobby-footer">
          <span>Powered by Rust + WebAssembly</span>
        </footer>
      </main>
    </div>
  );
}
