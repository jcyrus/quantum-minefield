"use client";

import { useState } from "react";
import { Lobby, type GameConfig } from "@/components/Lobby";
import { QuantumBoard } from "@/components/QuantumBoard";

export default function Page() {
  const [gameConfig, setGameConfig] = useState<GameConfig | null>(null);

  if (!gameConfig) {
    return <Lobby onStartGame={setGameConfig} />;
  }

  return (
    <QuantumBoard
      width={gameConfig.width}
      height={gameConfig.height}
      mineCount={gameConfig.mineCount}
      difficultyLabel={gameConfig.label}
      onBackToLobby={() => setGameConfig(null)}
    />
  );
}
