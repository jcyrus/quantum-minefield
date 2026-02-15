/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const __wbg_quantumcell_free: (a: number, b: number) => void;
export const __wbg_quantumgame_free: (a: number, b: number) => void;
export const init_game: (a: number, b: number, c: number, d: number, e: number) => number;
export const init_game_seeded: (a: number, b: number, c: number, d: bigint, e: number, f: number) => number;
export const quantumcell_probability: (a: number) => number;
export const quantumcell_state: (a: number) => [number, number];
export const quantumcell_x: (a: number) => number;
export const quantumcell_y: (a: number) => number;
export const quantumgame_apply_hadamard: (a: number, b: number, c: number) => [number, number, number];
export const quantumgame_contain_cell: (a: number, b: number, c: number) => [number, number, number];
export const quantumgame_get_cell: (a: number, b: number, c: number) => [number, number, number];
export const quantumgame_get_grid_snapshot: (a: number) => [number, number, number];
export const quantumgame_get_probability_cloud: (a: number) => [number, number, number];
export const quantumgame_get_seed: (a: number) => bigint;
export const quantumgame_is_quantum_inspector_enabled: (a: number) => number;
export const quantumgame_measure_weak: (a: number, b: number, c: number) => [number, number, number];
export const quantumgame_reveal_cell: (a: number, b: number, c: number) => [number, number, number];
export const quantumgame_set_quantum_inspector: (a: number, b: number) => void;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_start: () => void;
