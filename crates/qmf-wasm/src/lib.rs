use qmf_core::grid::{CellState, QuantumCell as CoreQuantumCell, QuantumGrid};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct QuantumCell {
    x: u32,
    y: u32,
    probability: f64,
    state: String,
}

#[wasm_bindgen]
impl QuantumCell {
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> u32 {
        self.x
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> u32 {
        self.y
    }

    #[wasm_bindgen(getter)]
    pub fn probability(&self) -> f64 {
        self.probability
    }

    #[wasm_bindgen(getter)]
    pub fn state(&self) -> String {
        self.state.clone()
    }
}

impl From<&CoreQuantumCell> for QuantumCell {
    fn from(value: &CoreQuantumCell) -> Self {
        match value.state {
            CellState::Superposition { probability } => Self {
                x: value.x,
                y: value.y,
                probability,
                state: "superposition".to_string(),
            },
            CellState::Revealed { .. } => Self {
                x: value.x,
                y: value.y,
                probability: 0.0,
                state: "revealed".to_string(),
            },
            CellState::Contained => Self {
                x: value.x,
                y: value.y,
                probability: 1.0,
                state: "contained".to_string(),
            },
            CellState::Detonated => Self {
                x: value.x,
                y: value.y,
                probability: 1.0,
                state: "detonated".to_string(),
            },
        }
    }
}

#[wasm_bindgen]
pub struct QuantumGame {
    grid: QuantumGrid,
    quantum_inspector_enabled: bool,
}

/// Create a new game with a random seed.
#[wasm_bindgen]
pub fn init_game(width: u32, height: u32, mine_count: u32, difficulty: &str) -> QuantumGame {
    // Generate a seed from JS Math.random (good enough for games)
    let raw = js_sys::Math::random();
    let seed = (raw * u64::MAX as f64) as u64;
    QuantumGame {
        grid: QuantumGrid::new(width, height, mine_count, seed, difficulty),
        quantum_inspector_enabled: false,
    }
}

/// Create a new game with an explicit seed (for replays / sharing).
#[wasm_bindgen]
pub fn init_game_seeded(
    width: u32,
    height: u32,
    mine_count: u32,
    seed: u64,
    difficulty: &str,
) -> QuantumGame {
    QuantumGame {
        grid: QuantumGrid::new(width, height, mine_count, seed, difficulty),
        quantum_inspector_enabled: false,
    }
}

#[wasm_bindgen]
impl QuantumGame {
    pub fn reveal_cell(&mut self, x: u32, y: u32) -> Result<JsValue, JsValue> {
        let outcome = self.grid.reveal_cell(x, y);
        to_js_value(&outcome)
    }

    pub fn contain_cell(&mut self, x: u32, y: u32) -> Result<JsValue, JsValue> {
        let outcome = self.grid.contain_cell(x, y);
        to_js_value(&outcome)
    }

    pub fn get_probability_cloud(&self) -> Result<JsValue, JsValue> {
        let cloud = self.grid.get_probability_cloud();
        to_js_value(&cloud)
    }

    pub fn get_grid_snapshot(&self) -> Result<JsValue, JsValue> {
        let snapshot = self.grid.snapshot();
        to_js_value(&snapshot)
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Result<QuantumCell, JsValue> {
        let index = if x < self.grid.width && y < self.grid.height {
            (y * self.grid.width + x) as usize
        } else {
            return Err(JsValue::from_str("coordinates out of bounds"));
        };

        Ok(QuantumCell::from(&self.grid.cells[index]))
    }

    pub fn get_seed(&self) -> u64 {
        self.grid.seed
    }

    pub fn set_quantum_inspector(&mut self, enabled: bool) {
        self.quantum_inspector_enabled = enabled;
    }

    pub fn is_quantum_inspector_enabled(&self) -> bool {
        self.quantum_inspector_enabled
    }
}

fn to_js_value<T>(value: &T) -> Result<JsValue, JsValue>
where
    T: serde::Serialize,
{
    let serializer =
        serde_wasm_bindgen::Serializer::new().serialize_large_number_types_as_bigints(true);
    value
        .serialize(&serializer)
        .map_err(|error| JsValue::from_str(&format!("serialization failure: {error}")))
}
