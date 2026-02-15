declare module "/wasm/qmf_wasm.js" {
  export default function init(): Promise<void>;
  export function init_game(
    width: number,
    height: number,
    mineCount: number,
    difficulty: string,
  ): import("@/types/quantum").WasmGame;
  export function init_game_seeded(
    width: number,
    height: number,
    mineCount: number,
    seed: bigint,
    difficulty: string,
  ): import("@/types/quantum").WasmGame;
}
