/* tslint:disable */
/* eslint-disable */

export class QuantumCell {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly probability: number;
    readonly state: string;
    readonly x: number;
    readonly y: number;
}

export class QuantumGame {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    contain_cell(x: number, y: number): any;
    get_cell(x: number, y: number): QuantumCell;
    get_grid_snapshot(): any;
    get_probability_cloud(): any;
    get_seed(): bigint;
    is_quantum_inspector_enabled(): boolean;
    reveal_cell(x: number, y: number): any;
    set_quantum_inspector(enabled: boolean): void;
}

/**
 * Create a new game with a random seed.
 */
export function init_game(width: number, height: number, mine_count: number, difficulty: string): QuantumGame;

/**
 * Create a new game with an explicit seed (for replays / sharing).
 */
export function init_game_seeded(width: number, height: number, mine_count: number, seed: bigint, difficulty: string): QuantumGame;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_quantumcell_free: (a: number, b: number) => void;
    readonly __wbg_quantumgame_free: (a: number, b: number) => void;
    readonly init_game: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly init_game_seeded: (a: number, b: number, c: number, d: bigint, e: number, f: number) => number;
    readonly quantumcell_probability: (a: number) => number;
    readonly quantumcell_state: (a: number) => [number, number];
    readonly quantumcell_x: (a: number) => number;
    readonly quantumcell_y: (a: number) => number;
    readonly quantumgame_contain_cell: (a: number, b: number, c: number) => [number, number, number];
    readonly quantumgame_get_cell: (a: number, b: number, c: number) => [number, number, number];
    readonly quantumgame_get_grid_snapshot: (a: number) => [number, number, number];
    readonly quantumgame_get_probability_cloud: (a: number) => [number, number, number];
    readonly quantumgame_get_seed: (a: number) => bigint;
    readonly quantumgame_is_quantum_inspector_enabled: (a: number) => number;
    readonly quantumgame_reveal_cell: (a: number, b: number, c: number) => [number, number, number];
    readonly quantumgame_set_quantum_inspector: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
