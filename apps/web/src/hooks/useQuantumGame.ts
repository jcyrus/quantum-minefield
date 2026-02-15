"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import type { GridSnapshot, WasmGame, WasmModule } from "@/types/quantum";

function isGridSnapshot(value: unknown): value is GridSnapshot {
  if (!value || typeof value !== "object") {
    return false;
  }

  const candidate = value as Partial<GridSnapshot>;
  return (
    typeof candidate.width === "number" &&
    typeof candidate.height === "number" &&
    typeof candidate.game_over === "boolean" &&
    typeof candidate.won === "boolean" &&
    typeof candidate.entropy === "number" &&
    typeof candidate.containment_charges === "number" &&
    Array.isArray(candidate.cells)
  );
}

export function useQuantumGame(
  width = 12,
  height = 12,
  mineCount = 24,
  difficulty = "researcher",
) {
  const gameRef = useRef<WasmGame | null>(null);
  const wasmRef = useRef<WasmModule | null>(null);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [inspectorEnabled, setInspectorEnabled] = useState(false);
  const [grid, setGrid] = useState<GridSnapshot | null>(null);
  const [probabilityCloud, setProbabilityCloud] = useState<number[]>([]);

  const disposeCurrentGame = useCallback(() => {
    const game = gameRef.current;
    if (game && typeof game.free === "function") {
      try {
        game.free();
      } catch {
        // ignored: stale/freed wasm object in dev hot-reload
      }
    }
    gameRef.current = null;
  }, []);

  const refresh = useCallback(() => {
    const game = gameRef.current;
    if (!game) {
      return;
    }

    try {
      const snapshot = game.get_grid_snapshot();
      if (isGridSnapshot(snapshot)) {
        setGrid(snapshot);
      }

      const cloud = game.get_probability_cloud();
      if (Array.isArray(cloud)) {
        setProbabilityCloud(
          cloud.filter((item): item is number => typeof item === "number"),
        );
      }
    } catch (refreshError) {
      setError(
        refreshError instanceof Error
          ? refreshError.message
          : "Wasm refresh failed",
      );
      setReady(false);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;

    async function boot() {
      try {
        disposeCurrentGame();

        // Load wasm glue via dynamic Function constructor to prevent
        // Turbopack from statically analysing and bundling the import.
        const load = new Function("url", "return import(url)") as (
          url: string,
        ) => Promise<WasmModule>;
        const cacheBuster = Date.now();
        const wasmModule = await load(`/wasm/qmf_wasm.js?v=${cacheBuster}`);
        await wasmModule.default(`/wasm/qmf_wasm_bg.wasm?v=${cacheBuster}`);
        wasmRef.current = wasmModule;

        if (cancelled) {
          return;
        }

        gameRef.current = wasmRef.current.init_game(
          width,
          height,
          mineCount,
          difficulty,
        );
        setInspectorEnabled(gameRef.current.is_quantum_inspector_enabled());
        setError(null);
        setReady(true);
        refresh();
      } catch (bootError) {
        if (cancelled) {
          return;
        }

        setError(
          bootError instanceof Error
            ? bootError.message
            : "Unable to initialize wasm game",
        );
      }
    }

    boot();

    return () => {
      cancelled = true;
      disposeCurrentGame();
    };
  }, [difficulty, disposeCurrentGame, height, mineCount, refresh, width]);

  const revealCell = useCallback(
    (x: number, y: number) => {
      const game = gameRef.current;
      if (!game || !ready) {
        return;
      }

      try {
        game.reveal_cell(x, y);
        refresh();
      } catch (revealError) {
        setError(
          revealError instanceof Error
            ? revealError.message
            : "Reveal action failed",
        );
        setReady(false);
      }
    },
    [ready, refresh],
  );

  const containCell = useCallback(
    (x: number, y: number) => {
      const game = gameRef.current;
      if (!game || !ready) {
        return;
      }

      try {
        game.contain_cell(x, y);
        refresh();
      } catch (containError) {
        setError(
          containError instanceof Error
            ? containError.message
            : "Containment action failed",
        );
        setReady(false);
      }
    },
    [ready, refresh],
  );

  const toggleInspector = useCallback(() => {
    const game = gameRef.current;
    if (!game) {
      return;
    }

    const nextValue = !game.is_quantum_inspector_enabled();
    game.set_quantum_inspector(nextValue);
    setInspectorEnabled(nextValue);
    refresh();
  }, [refresh]);

  const newGame = useCallback(() => {
    const wasm = wasmRef.current;
    if (!wasm) return;
    disposeCurrentGame();
    try {
      gameRef.current = wasm.init_game(width, height, mineCount, difficulty);
      setInspectorEnabled(false);
      setError(null);
      setReady(true);
      refresh();
    } catch (newGameError) {
      setError(
        newGameError instanceof Error
          ? newGameError.message
          : "Failed to start a new game",
      );
      setReady(false);
    }
  }, [disposeCurrentGame, width, height, mineCount, difficulty, refresh]);

  return {
    ready,
    error,
    inspectorEnabled,
    grid,
    probabilityCloud,
    revealCell,
    containCell,
    toggleInspector,
    newGame,
  };
}
