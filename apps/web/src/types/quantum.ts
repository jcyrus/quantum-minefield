export type CellState =
  | "superposition"
  | "revealed"
  | "contained"
  | "detonated";

export interface QuantumCellView {
  x: number;
  y: number;
  state: {
    state: CellState;
    probability?: number;
    adjacent_mines?: number;
  };
}

export interface GridSnapshot {
  width: number;
  height: number;
  game_over: boolean;
  won: boolean;
  seed: bigint;
  containment_charges: number;
  entropy: number;
  cells: QuantumCellView[];
}

export interface WasmGame {
  free?: () => void;
  reveal_cell: (x: number, y: number) => unknown;
  contain_cell: (x: number, y: number) => unknown;
  get_grid_snapshot: () => unknown;
  get_probability_cloud: () => unknown;
  get_seed: () => number;
  set_quantum_inspector: (enabled: boolean) => void;
  is_quantum_inspector_enabled: () => boolean;
}

export interface WasmModule {
  default: (moduleOrPath?: unknown) => Promise<unknown>;
  init_game: (
    width: number,
    height: number,
    mineCount: number,
    difficulty: string,
  ) => WasmGame;
  init_game_seeded: (
    width: number,
    height: number,
    mineCount: number,
    seed: bigint,
    difficulty: string,
  ) => WasmGame;
}
