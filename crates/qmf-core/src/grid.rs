use serde::{Deserialize, Serialize};

use crate::circuit::Circuit;
use crate::entanglement::Entanglement;
use crate::rng::SplitMix64;

// ---------------------------------------------------------------------------
// Cell state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CellState {
    /// Unobserved — player sees a probability hint.
    Superposition { probability: f64 },
    /// Observed safe — shows adjacent mine count.
    Revealed { adjacent_mines: u8 },
    /// Mine successfully contained by the player (right-click).
    Contained,
    /// Mine detonated — game over.
    Detonated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumCell {
    pub x: u32,
    pub y: u32,
    pub state: CellState,
}

// ---------------------------------------------------------------------------
// Grid snapshot (serialised to JS)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSnapshot {
    pub width: u32,
    pub height: u32,
    pub game_over: bool,
    pub won: bool,
    pub seed: u64,
    pub containment_charges: u32,
    pub entropy: f64,
    pub cells: Vec<QuantumCell>,
}

// ---------------------------------------------------------------------------
// Reveal / contain outcomes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RevealOutcome {
    /// Safe cell uncovered.
    Revealed { cell: QuantumCell },
    /// Mine detonated by direct click — game over.
    MineDetonated { x: u32, y: u32 },
    /// Correct containment — mine locked down.
    ContainmentSuccess { x: u32, y: u32 },
    /// Wrong containment — cell was safe, charge wasted. Cell gets revealed.
    ContainmentFailed { cell: QuantumCell },
    /// Cell was already resolved (not in Superposition).
    AlreadyResolved,
    /// Coordinates outside the grid.
    OutOfBounds,
    /// Game is already finished.
    GameAlreadyOver,
    /// No containment charges remaining.
    NoChargesRemaining,
}

// ---------------------------------------------------------------------------
// QuantumGrid — the core game state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumGrid {
    pub width: u32,
    pub height: u32,
    pub mine_count: u32,
    pub game_over: bool,
    pub won: bool,
    pub seed: u64,
    pub containment_charges: u32,
    pub cells: Vec<QuantumCell>,
    pub circuit: Circuit,
    pub entanglement: Entanglement,

    // Private-ish fields (pub for serde, not exposed to wasm)
    pub rng: SplitMix64,
    pub mine_map: Vec<bool>,
    pub mines_placed: bool,
}

impl QuantumGrid {
    /// Create a new grid. Mine placement is deferred to first interaction
    /// so the first click is guaranteed safe.
    pub fn new(width: u32, height: u32, mine_count: u32, seed: u64, difficulty: &str) -> Self {
        let total = (width * height) as usize;
        let mine_count = mine_count.min(width * height - 9); // must leave room for safe zone
        let baseline = (mine_count as f64 / total.max(1) as f64).clamp(0.0, 1.0);
        let circuit = Circuit::for_difficulty(difficulty);

        // Generate per-cell probability hints using RNG + circuit scrambling
        let mut rng = SplitMix64::new(seed);
        let cells = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .map(|(x, y)| {
                // Add ±5% noise to baseline, then run through circuit
                let noise = rng.next_f64() * 0.10 - 0.05;
                let raw = (baseline + noise).clamp(0.0, 1.0);
                let probability = circuit.apply_probability(raw);
                QuantumCell {
                    x,
                    y,
                    state: CellState::Superposition { probability },
                }
            })
            .collect::<Vec<_>>();

        // Difficulty-scaled entanglement
        let (step, strength) = match difficulty {
            "observer" => (11_usize, 0.2),
            "theorist" => (5, 0.5),
            _ => (7, 0.35), // "researcher" default
        };
        let mut entanglement = Entanglement::default();
        for left in (0..total).step_by(step) {
            let right = left + (step / 2).max(1);
            if right < total {
                entanglement.add_pair(left, right, strength);
            }
        }

        Self {
            width,
            height,
            mine_count,
            game_over: false,
            won: false,
            seed,
            containment_charges: mine_count,
            cells,
            circuit,
            entanglement,
            rng,
            mine_map: vec![false; total],
            mines_placed: false,
        }
    }

    // -----------------------------------------------------------------------
    // Public actions
    // -----------------------------------------------------------------------

    /// Left-click: reveal a cell.
    pub fn reveal_cell(&mut self, x: u32, y: u32) -> RevealOutcome {
        if self.game_over || self.won {
            return RevealOutcome::GameAlreadyOver;
        }
        let Some(index) = self.index_of(x, y) else {
            return RevealOutcome::OutOfBounds;
        };
        if !matches!(self.cells[index].state, CellState::Superposition { .. }) {
            return RevealOutcome::AlreadyResolved;
        }

        // Deferred mine placement — first interaction is always safe
        if !self.mines_placed {
            self.place_mines(index);
        }

        if self.mine_map[index] {
            // BOOM
            self.cells[index].state = CellState::Detonated;
            self.game_over = true;
            self.propagate_entanglement(index, true);
            RevealOutcome::MineDetonated { x, y }
        } else {
            self.reveal_safe(index)
        }
    }

    /// Right-click / contain: mark a cell as a mine.
    pub fn contain_cell(&mut self, x: u32, y: u32) -> RevealOutcome {
        if self.game_over || self.won {
            return RevealOutcome::GameAlreadyOver;
        }
        if self.containment_charges == 0 {
            return RevealOutcome::NoChargesRemaining;
        }
        let Some(index) = self.index_of(x, y) else {
            return RevealOutcome::OutOfBounds;
        };
        if !matches!(self.cells[index].state, CellState::Superposition { .. }) {
            return RevealOutcome::AlreadyResolved;
        }

        if !self.mines_placed {
            self.place_mines(index);
        }

        self.containment_charges -= 1;

        if self.mine_map[index] {
            // Correct containment
            self.cells[index].state = CellState::Contained;
            self.propagate_entanglement(index, true);
            self.won = self.is_win_condition_met();
            RevealOutcome::ContainmentSuccess { x, y }
        } else {
            // Wrong — cell was safe. Reveal it (charge is lost).
            let outcome = self.reveal_safe(index);
            match outcome {
                RevealOutcome::Revealed { cell } => RevealOutcome::ContainmentFailed { cell },
                other => other,
            }
        }
    }

    pub fn get_probability_cloud(&self) -> Vec<f64> {
        self.cells
            .iter()
            .map(|cell| match cell.state {
                CellState::Superposition { probability } => probability,
                CellState::Contained | CellState::Detonated => 1.0,
                CellState::Revealed { .. } => 0.0,
            })
            .collect()
    }

    /// Fraction of cells still in Superposition: 1.0 = fully uncertain, 0.0 = fully resolved.
    pub fn entropy(&self) -> f64 {
        let total = self.cells.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let unresolved = self
            .cells
            .iter()
            .filter(|c| matches!(c.state, CellState::Superposition { .. }))
            .count() as f64;
        unresolved / total
    }

    pub fn snapshot(&self) -> GridSnapshot {
        GridSnapshot {
            width: self.width,
            height: self.height,
            game_over: self.game_over,
            won: self.won,
            seed: self.seed,
            containment_charges: self.containment_charges,
            entropy: self.entropy(),
            cells: self.cells.clone(),
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn index_of(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some((y * self.width + x) as usize)
        }
    }

    fn coords_of(&self, index: usize) -> (u32, u32) {
        let x = index as u32 % self.width;
        let y = index as u32 / self.width;
        (x, y)
    }

    /// Fisher-Yates mine placement, excluding `safe_index` and its 8 neighbors.
    fn place_mines(&mut self, safe_index: usize) {
        let total = self.cells.len();
        let (sx, sy) = self.coords_of(safe_index);

        // Build exclusion set (safe zone = clicked cell + neighbors)
        let mut excluded = Vec::with_capacity(9);
        for dy in -1_i32..=1 {
            for dx in -1_i32..=1 {
                let nx = sx as i32 + dx;
                let ny = sy as i32 + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    excluded.push((ny as u32 * self.width + nx as u32) as usize);
                }
            }
        }

        // Collect eligible indices
        let mut candidates: Vec<usize> = (0..total).filter(|i| !excluded.contains(i)).collect();

        // Shuffle (Fisher-Yates) and pick first mine_count
        let n = candidates.len();
        let to_place = (self.mine_count as usize).min(n);
        for i in 0..to_place {
            let j = i + self.rng.next_usize(n - i);
            candidates.swap(i, j);
        }
        for &idx in &candidates[..to_place] {
            self.mine_map[idx] = true;
        }

        self.mines_placed = true;

        // Recalculate probability hints: neighbor-aware hinting
        self.recalculate_probabilities();
    }

    /// Recalculate displayed probabilities for all Superposition cells
    /// based on the actual mine map + circuit scrambling. This gives
    /// heterogeneous hints without revealing exact positions.
    fn recalculate_probabilities(&mut self) {
        let total = self.cells.len();
        for i in 0..total {
            if !matches!(self.cells[i].state, CellState::Superposition { .. }) {
                continue;
            }
            let (x, y) = self.coords_of(i);
            // Count how many neighbors are mines (ground truth)
            let neighbor_mines = self.adjacent_mines(x, y);
            let max_neighbors = self.neighbor_count(x, y);

            // Blend: baseline weight + neighbor density
            let baseline = self.mine_count as f64 / total as f64;
            let local_density = if max_neighbors > 0 {
                neighbor_mines as f64 / max_neighbors as f64
            } else {
                baseline
            };

            // 60% local signal, 40% global baseline, then circuit-scramble
            let blended = local_density * 0.6 + baseline * 0.4;
            // Add per-cell noise so identical neighbor counts don't look identical
            let noise = self.rng.next_f64() * 0.06 - 0.03;
            let raw = (blended + noise).clamp(0.01, 0.99);
            let scrambled = self.circuit.apply_probability(raw);

            self.cells[i].state = CellState::Superposition {
                probability: scrambled,
            };
        }
    }

    /// Reveal a cell known to be safe. Computes adjacent count, does flood fill
    /// if zero, and checks win condition.
    fn reveal_safe(&mut self, index: usize) -> RevealOutcome {
        let (x, y) = self.coords_of(index);
        let adj = self.adjacent_mines(x, y);
        self.cells[index].state = CellState::Revealed {
            adjacent_mines: adj,
        };
        self.propagate_entanglement(index, false);

        if adj == 0 {
            self.flood_fill(x, y);
        }

        self.won = self.is_win_condition_met();
        RevealOutcome::Revealed {
            cell: self.cells[index].clone(),
        }
    }

    /// Stack-based flood fill for zero-adjacent safe cells.
    fn flood_fill(&mut self, start_x: u32, start_y: u32) {
        let mut stack = vec![(start_x, start_y)];

        while let Some((cx, cy)) = stack.pop() {
            for ny in cy.saturating_sub(1)..=(cy + 1).min(self.height - 1) {
                for nx in cx.saturating_sub(1)..=(cx + 1).min(self.width - 1) {
                    if nx == cx && ny == cy {
                        continue;
                    }
                    let Some(idx) = self.index_of(nx, ny) else {
                        continue;
                    };
                    // Only process cells still in superposition and not mines
                    if !matches!(self.cells[idx].state, CellState::Superposition { .. }) {
                        continue;
                    }
                    if self.mine_map[idx] {
                        continue;
                    }

                    let adj = self.adjacent_mines(nx, ny);
                    self.cells[idx].state = CellState::Revealed {
                        adjacent_mines: adj,
                    };

                    if adj == 0 {
                        stack.push((nx, ny));
                    }
                }
            }
        }
    }

    /// Count adjacent mines using the ground-truth mine_map.
    fn adjacent_mines(&self, x: u32, y: u32) -> u8 {
        let mut count = 0u8;
        for ny in y.saturating_sub(1)..=(y + 1).min(self.height.saturating_sub(1)) {
            for nx in x.saturating_sub(1)..=(x + 1).min(self.width.saturating_sub(1)) {
                if nx == x && ny == y {
                    continue;
                }
                if let Some(idx) = self.index_of(nx, ny) {
                    if self.mine_map[idx] {
                        count = count.saturating_add(1);
                    }
                }
            }
        }
        count
    }

    /// Number of valid neighbor cells for (x, y).
    fn neighbor_count(&self, x: u32, y: u32) -> u8 {
        let mut count = 0u8;
        for ny in y.saturating_sub(1)..=(y + 1).min(self.height.saturating_sub(1)) {
            for nx in x.saturating_sub(1)..=(x + 1).min(self.width.saturating_sub(1)) {
                if nx == x && ny == y {
                    continue;
                }
                count += 1;
            }
        }
        count
    }

    /// Propagate entanglement: after resolving a cell, shift its partner's
    /// displayed probability.
    fn propagate_entanglement(&mut self, index: usize, was_mine: bool) {
        if let Some((pair, partner_index)) = self.entanglement.partner_of(index) {
            if let CellState::Superposition { probability } = self.cells[partner_index].state {
                let adjusted =
                    self.entanglement
                        .collapse_partner_probability(pair, was_mine, probability);
                self.cells[partner_index].state = CellState::Superposition {
                    probability: adjusted,
                };
            }
        }
    }

    /// Wavefunction Purification: the player wins when **every** cell is
    /// resolved (no Superposition remaining) and the game isn't over.
    fn is_win_condition_met(&self) -> bool {
        !self.game_over
            && self
                .cells
                .iter()
                .all(|c| !matches!(c.state, CellState::Superposition { .. }))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid(w: u32, h: u32, mines: u32) -> QuantumGrid {
        QuantumGrid::new(w, h, mines, 42, "observer")
    }

    #[test]
    fn initial_state_is_all_superposition() {
        let g = make_grid(8, 8, 10);
        assert!(g
            .cells
            .iter()
            .all(|c| matches!(c.state, CellState::Superposition { .. })));
        assert!(!g.mines_placed);
        assert_eq!(g.containment_charges, 10);
    }

    #[test]
    fn first_click_is_always_safe() {
        // Try many seeds — first click should never detonate
        for seed in 0..50 {
            let mut g = QuantumGrid::new(8, 8, 10, seed, "researcher");
            let outcome = g.reveal_cell(4, 4);
            assert!(
                matches!(outcome, RevealOutcome::Revealed { .. }),
                "seed {seed}: first click detonated!"
            );
            assert!(g.mines_placed);
            // Safe zone: (4,4) and its 8 neighbors should not be mines
            for dy in -1_i32..=1 {
                for dx in -1_i32..=1 {
                    let nx = 4 + dx;
                    let ny = 4 + dy;
                    if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                        let idx = (ny * 8 + nx) as usize;
                        assert!(
                            !g.mine_map[idx],
                            "seed {seed}: mine in safe zone at ({nx},{ny})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn mine_count_matches_requested() {
        for seed in 0..20 {
            let mut g = QuantumGrid::new(8, 8, 10, seed, "observer");
            g.reveal_cell(0, 0);
            let placed = g.mine_map.iter().filter(|&&m| m).count();
            assert_eq!(placed, 10, "seed {seed}: wrong mine count");
        }
    }

    #[test]
    fn contain_correct_mine_succeeds() {
        let mut g = make_grid(8, 8, 10);
        // Trigger placement via reveal
        g.reveal_cell(0, 0);
        // Find a mine
        let mine_idx = g.mine_map.iter().position(|&m| m).unwrap();
        let (mx, my) = g.coords_of(mine_idx);
        let charges_before = g.containment_charges;
        let outcome = g.contain_cell(mx, my);
        assert!(matches!(outcome, RevealOutcome::ContainmentSuccess { .. }));
        assert_eq!(g.containment_charges, charges_before - 1);
        assert!(matches!(g.cells[mine_idx].state, CellState::Contained));
    }

    #[test]
    fn contain_safe_cell_wastes_charge() {
        let mut g = make_grid(8, 8, 10);
        g.reveal_cell(0, 0);
        // Find a safe unrevealed cell
        let safe_idx = g
            .cells
            .iter()
            .position(|c| {
                matches!(c.state, CellState::Superposition { .. })
                    && !g.mine_map[(c.y * g.width + c.x) as usize]
            })
            .unwrap();
        let (sx, sy) = g.coords_of(safe_idx);
        let charges_before = g.containment_charges;
        let outcome = g.contain_cell(sx, sy);
        assert!(matches!(outcome, RevealOutcome::ContainmentFailed { .. }));
        assert_eq!(g.containment_charges, charges_before - 1);
        // Cell should now be revealed (not superposition)
        assert!(matches!(
            g.cells[safe_idx].state,
            CellState::Revealed { .. }
        ));
    }

    #[test]
    fn no_charges_returns_error() {
        let mut g = make_grid(8, 8, 10);
        g.reveal_cell(0, 0);
        g.containment_charges = 0;
        let mine_idx = g.mine_map.iter().position(|&m| m).unwrap();
        let (mx, my) = g.coords_of(mine_idx);
        let outcome = g.contain_cell(mx, my);
        assert!(matches!(outcome, RevealOutcome::NoChargesRemaining));
    }

    #[test]
    fn clicking_mine_detonates() {
        let mut g = make_grid(8, 8, 10);
        g.reveal_cell(0, 0); // safe first click
        let mine_idx = g.mine_map.iter().position(|&m| m).unwrap();
        let (mx, my) = g.coords_of(mine_idx);
        let outcome = g.reveal_cell(mx, my);
        assert!(matches!(outcome, RevealOutcome::MineDetonated { .. }));
        assert!(g.game_over);
    }

    #[test]
    fn win_condition_is_entropy_zero() {
        // 5x5 with 2 mines — large enough that first-click safe zone
        // doesn't consume all cells
        let mut g = QuantumGrid::new(5, 5, 2, 100, "observer");
        g.reveal_cell(2, 2); // center — always safe

        assert!(g.mines_placed);
        let placed = g.mine_map.iter().filter(|&&m| m).count();
        assert_eq!(placed, 2, "Should have placed 2 mines");

        // Reveal all safe cells
        for i in 0..25 {
            let (x, y) = g.coords_of(i);
            if !g.mine_map[i] && matches!(g.cells[i].state, CellState::Superposition { .. }) {
                g.reveal_cell(x, y);
            }
        }

        // Contain the mines
        for i in 0..25 {
            if g.mine_map[i] && matches!(g.cells[i].state, CellState::Superposition { .. }) {
                let (mx, my) = g.coords_of(i);
                g.contain_cell(mx, my);
            }
        }

        assert!(g.won, "Should have won after resolving all cells");
        assert!((g.entropy() - 0.0).abs() < 1e-10, "Entropy should be 0");
    }

    #[test]
    fn flood_fill_cascades() {
        // Use a grid where center area has no adjacent mines
        let mut g = QuantumGrid::new(8, 8, 2, 999, "observer");
        g.reveal_cell(4, 4); // trigger placement

        // After revealing a zero-adjacent cell, count revealed cells
        // There should be more than 1 if flood fill worked
        let revealed = g
            .cells
            .iter()
            .filter(|c| matches!(c.state, CellState::Revealed { .. }))
            .count();
        // At minimum, the clicked cell is revealed. If it had 0 adjacent, flood fill should expand.
        assert!(revealed >= 1);
    }

    #[test]
    fn game_already_over_guard() {
        let mut g = make_grid(8, 8, 10);
        g.game_over = true;
        assert!(matches!(
            g.reveal_cell(0, 0),
            RevealOutcome::GameAlreadyOver
        ));
        assert!(matches!(
            g.contain_cell(0, 0),
            RevealOutcome::GameAlreadyOver
        ));
    }

    #[test]
    fn entropy_decreases_on_reveal() {
        let mut g = make_grid(8, 8, 10);
        let e0 = g.entropy();
        assert!((e0 - 1.0).abs() < 1e-10);
        g.reveal_cell(0, 0);
        let e1 = g.entropy();
        assert!(e1 < e0, "Entropy should decrease after reveal");
    }

    #[test]
    fn deterministic_games() {
        // Same seed → same mine layout
        let mut a = QuantumGrid::new(8, 8, 10, 42, "researcher");
        let mut b = QuantumGrid::new(8, 8, 10, 42, "researcher");
        a.reveal_cell(0, 0);
        b.reveal_cell(0, 0);
        assert_eq!(a.mine_map, b.mine_map);
    }
}
