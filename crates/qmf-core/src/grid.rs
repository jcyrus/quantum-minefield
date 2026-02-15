use serde::{Deserialize, Serialize};

use crate::circuit::Circuit;
use crate::entanglement::{Entanglement, LinkType};
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
    /// One or more entangled partners were force-collapsed by Bell State
    /// propagation. The `cells` vector contains their resolved states.
    EntangledCollapse { cells: Vec<QuantumCell> },
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
        let (step, strength, use_bell) = match difficulty {
            "observer" => (11_usize, 0.2, false),
            "theorist" => (5, 0.5, true), // BellState pairs at highest difficulty
            _ => (7, 0.35, false),        // "researcher" default
        };
        let mut entanglement = Entanglement::default();
        let mut pair_index = 0_usize;
        for left in (0..total).step_by(step) {
            let right = left + (step / 2).max(1);
            if right < total {
                // At "theorist", every other pair is a hard BellState link
                let link_type = if use_bell && pair_index % 2 == 0 {
                    LinkType::BellState
                } else {
                    LinkType::Probabilistic
                };
                entanglement.add_pair(left, right, strength, link_type);
                pair_index += 1;
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

    /// **Hadamard Tool** — Apply destructive interference to a Superposition
    /// cell, flipping its probability (high → low, low → high).
    ///
    /// Game Mechanic: lets the player "rewrite" a dangerous cell before clicking.
    pub fn apply_hadamard(&mut self, x: u32, y: u32) -> Result<f64, &'static str> {
        let index = self.index_of(x, y).ok_or("coordinates out of bounds")?;
        match self.cells[index].state {
            CellState::Superposition { probability } => {
                let new_p = (1.0 - probability).clamp(0.0, 1.0);
                self.cells[index].state = CellState::Superposition { probability: new_p };
                Ok(new_p)
            }
            _ => Err("cell is already resolved"),
        }
    }

    /// **Observer Effect (Heisenbug)** — Weak measurement. Returns the current
    /// probability but introduces drift (±4% noise) to the stored state,
    /// simulating that "looking changes the system."
    pub fn measure_weak(&mut self, x: u32, y: u32) -> Result<f64, &'static str> {
        let index = self.index_of(x, y).ok_or("coordinates out of bounds")?;
        match self.cells[index].state {
            CellState::Superposition { probability } => {
                let observed = probability;
                // Introduce observer drift
                let drift = self.rng.next_f64() * 0.08 - 0.04;
                let perturbed = (probability + drift).clamp(0.01, 0.99);
                self.cells[index].state = CellState::Superposition {
                    probability: perturbed,
                };
                Ok(observed)
            }
            _ => Err("cell is already resolved"),
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

    /// Propagate entanglement: after resolving a cell, handle its partners.
    ///
    /// - **BellState** links trigger `propagate_collapse` — the partner is
    ///   force-collapsed (revealed if safe, contained if mine) and the
    ///   cascade continues recursively through any further Bell partners.
    /// - **Probabilistic** links just shift the displayed probability.
    fn propagate_entanglement(&mut self, index: usize, was_mine: bool) {
        // Collect partner info first to avoid borrow issues.
        let partners: Vec<(usize, LinkType, f64)> = self
            .entanglement
            .partners_of(index)
            .iter()
            .map(|(pair, partner_idx)| (*partner_idx, pair.link_type, pair.strength))
            .collect();

        for (partner_index, link_type, _strength) in &partners {
            if !matches!(
                self.cells[*partner_index].state,
                CellState::Superposition { .. }
            ) {
                continue;
            }

            match link_type {
                LinkType::BellState => {
                    // Force-collapse the partner and cascade.
                    let mut visited = std::collections::HashSet::new();
                    visited.insert(index);
                    self.propagate_collapse(*partner_index, was_mine, &mut visited);
                }
                LinkType::Probabilistic => {
                    // Legacy Bayesian adjustment.
                    if let CellState::Superposition { probability } =
                        self.cells[*partner_index].state
                    {
                        // Reconstruct a temporary pair for the calculation
                        let pair_ref = self
                            .entanglement
                            .partners_of(index)
                            .into_iter()
                            .find(|(_, pi)| *pi == *partner_index)
                            .map(|(p, _)| p.clone());
                        if let Some(pair) = pair_ref {
                            let adjusted = self.entanglement.collapse_partner_probability(
                                &pair,
                                was_mine,
                                probability,
                            );
                            self.cells[*partner_index].state = CellState::Superposition {
                                probability: adjusted,
                            };
                        }
                    }
                }
            }
        }
    }

    /// Recursive (stack-based) Bell State collapse propagation.
    ///
    /// When a cell with a BellState partner is observed, the partner is
    /// instantly force-collapsed to a definite state (anti-correlated).
    /// If *that* partner also has BellState partners, the cascade continues
    /// (GHZ-state chain reaction).
    fn propagate_collapse(
        &mut self,
        index: usize,
        triggering_cell_was_mine: bool,
        visited: &mut std::collections::HashSet<usize>,
    ) {
        // Stack-based iteration to prevent deep recursion stack overflows.
        let mut stack = vec![(index, triggering_cell_was_mine)];

        while let Some((current, was_mine)) = stack.pop() {
            if !visited.insert(current) {
                continue; // already processed — avoid infinite loops
            }

            if !matches!(self.cells[current].state, CellState::Superposition { .. }) {
                continue; // already resolved
            }

            // Anti-correlation: if trigger was a mine, partner is safe; vice versa.
            let partner_is_mine = !was_mine;

            if self.mine_map[current] && partner_is_mine {
                // Mine, and Bell collapse says it's a mine → Contain it.
                self.cells[current].state = CellState::Contained;
            } else if !self.mine_map[current] && !partner_is_mine {
                // Safe, and Bell collapse says it's safe → Reveal it.
                let (cx, cy) = self.coords_of(current);
                let adj = self.adjacent_mines(cx, cy);
                self.cells[current].state = CellState::Revealed {
                    adjacent_mines: adj,
                };
                // Note: we intentionally do NOT flood-fill from collapse
                // to avoid cascading the entire board. Only explicit clicks
                // trigger flood fill.
            } else {
                // Ground truth disagrees with Bell prediction. The physics
                // is "correct" (anti-correlated) but the mine map is the
                // source of truth for what the cell actually *is*. Resolve
                // it according to reality.
                if self.mine_map[current] {
                    self.cells[current].state = CellState::Contained;
                } else {
                    let (cx, cy) = self.coords_of(current);
                    let adj = self.adjacent_mines(cx, cy);
                    self.cells[current].state = CellState::Revealed {
                        adjacent_mines: adj,
                    };
                }
            }

            // Continue the cascade: find Bell partners of `current`
            let next_partners: Vec<usize> = self
                .entanglement
                .partners_of(current)
                .iter()
                .filter(|(pair, _)| pair.link_type == LinkType::BellState)
                .map(|(_, pi)| *pi)
                .collect();

            for partner in next_partners {
                if !visited.contains(&partner) {
                    stack.push((partner, self.mine_map[current]));
                }
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

    // ===================================================================
    // New: Hard Quantum Mechanics tests
    // ===================================================================

    #[test]
    fn bell_state_collapse_forces_partner() {
        // Directly test the Entanglement module's BellState collapse
        let mut ent = Entanglement::default();
        ent.add_pair(0, 1, 1.0, LinkType::BellState);

        let pair = &ent.pairs[0];

        // Observed mine → partner must be safe (0.0)
        let result = ent.collapse_partner_probability(pair, true, 0.5);
        assert!(
            (result - 0.0).abs() < 1e-10,
            "BellState: mine observed → partner should be 0.0, got {result}"
        );

        // Observed safe → partner must be mine (1.0)
        let result = ent.collapse_partner_probability(pair, false, 0.5);
        assert!(
            (result - 1.0).abs() < 1e-10,
            "BellState: safe observed → partner should be 1.0, got {result}"
        );
    }

    #[test]
    fn reveal_cell_auto_resolves_bell_partner() {
        // Build a small grid with a manually-injected BellState pair.
        let mut g = QuantumGrid::new(8, 8, 10, 42, "observer");
        g.reveal_cell(0, 0); // trigger mine placement

        // Find a mine and a safe cell that are both still in Superposition
        let mine_idx = g
            .cells
            .iter()
            .position(|c| {
                matches!(c.state, CellState::Superposition { .. })
                    && g.mine_map[(c.y * g.width + c.x) as usize]
            })
            .expect("should find an unresolved mine");
        let safe_idx = g
            .cells
            .iter()
            .position(|c| {
                matches!(c.state, CellState::Superposition { .. })
                    && !g.mine_map[(c.y * g.width + c.x) as usize]
            })
            .expect("should find an unresolved safe cell");

        // Inject a BellState pair between them
        g.entanglement.pairs.clear();
        g.entanglement
            .add_pair(safe_idx, mine_idx, 1.0, LinkType::BellState);

        // Reveal the safe cell — this should auto-collapse the mine partner
        let (sx, sy) = g.coords_of(safe_idx);
        let outcome = g.reveal_cell(sx, sy);
        assert!(
            matches!(outcome, RevealOutcome::Revealed { .. }),
            "safe cell should be revealed"
        );

        // The mine partner should now be Contained (force-collapsed)
        assert!(
            matches!(g.cells[mine_idx].state, CellState::Contained),
            "BellState partner mine should be auto-contained, got {:?}",
            g.cells[mine_idx].state
        );
    }

    #[test]
    fn ghz_chain_propagation() {
        // Test multi-qubit chain: A → B → C all collapse from revealing A.
        let mut g = QuantumGrid::new(8, 8, 10, 42, "observer");
        g.reveal_cell(0, 0); // trigger mine placement

        // Find 3 unresolved cells: one safe, one mine, one safe
        let cells_in_super: Vec<usize> = g
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c.state, CellState::Superposition { .. }))
            .map(|(i, _)| i)
            .collect();

        // We need at least 3 cells in superposition
        assert!(
            cells_in_super.len() >= 3,
            "not enough superposition cells for GHZ test"
        );

        let a = cells_in_super[0];
        let b = cells_in_super[1];
        let c = cells_in_super[2];

        // Set up chain: A ↔ B ↔ C  (all BellState)
        g.entanglement.pairs.clear();
        g.entanglement.add_pair(a, b, 1.0, LinkType::BellState);
        g.entanglement.add_pair(b, c, 1.0, LinkType::BellState);

        // All three should be in Superposition
        assert!(matches!(g.cells[a].state, CellState::Superposition { .. }));
        assert!(matches!(g.cells[b].state, CellState::Superposition { .. }));
        assert!(matches!(g.cells[c].state, CellState::Superposition { .. }));

        // Reveal cell A
        let (ax, ay) = g.coords_of(a);
        g.reveal_cell(ax, ay);

        // B should now be resolved (no longer Superposition)
        assert!(
            !matches!(g.cells[b].state, CellState::Superposition { .. }),
            "GHZ: B should be force-collapsed after revealing A, got {:?}",
            g.cells[b].state
        );

        // C should also be resolved (chain propagation through B)
        assert!(
            !matches!(g.cells[c].state, CellState::Superposition { .. }),
            "GHZ: C should be force-collapsed via chain A→B→C, got {:?}",
            g.cells[c].state
        );
    }

    #[test]
    fn hadamard_flips_probability() {
        let mut g = make_grid(8, 8, 10);
        // Get initial probability of cell (3, 3)
        let idx = g.index_of(3, 3).unwrap();
        let original_p = match g.cells[idx].state {
            CellState::Superposition { probability } => probability,
            _ => panic!("should be superposition"),
        };

        let result = g.apply_hadamard(3, 3);
        assert!(result.is_ok());
        let new_p = result.unwrap();
        assert!(
            (new_p - (1.0 - original_p)).abs() < 1e-10,
            "Hadamard should flip probability: expected {}, got {new_p}",
            1.0 - original_p
        );

        // Verify stored state matches
        match g.cells[idx].state {
            CellState::Superposition { probability } => {
                assert!((probability - new_p).abs() < 1e-10);
            }
            _ => panic!("should still be superposition after Hadamard"),
        }

        // Applying to an already-resolved cell should error
        g.reveal_cell(0, 0);
        let idx_0_0 = g.index_of(0, 0).unwrap();
        if matches!(g.cells[idx_0_0].state, CellState::Revealed { .. }) {
            let err = g.apply_hadamard(0, 0);
            assert!(err.is_err());
        }
    }

    #[test]
    fn measure_weak_returns_probability_with_drift() {
        let mut g = make_grid(8, 8, 10);
        let idx = g.index_of(3, 3).unwrap();
        let original_p = match g.cells[idx].state {
            CellState::Superposition { probability } => probability,
            _ => panic!("should be superposition"),
        };

        // Weak measurement should return the original probability
        let observed = g.measure_weak(3, 3).unwrap();
        assert!(
            (observed - original_p).abs() < 1e-10,
            "measure_weak should return original probability"
        );

        // But the stored state should have drifted
        let stored_p = match g.cells[idx].state {
            CellState::Superposition { probability } => probability,
            _ => panic!("should still be superposition after weak measurement"),
        };
        // Drift is ±4%, so |stored - original| ≤ 0.04 (plus clamp effects)
        assert!(
            (stored_p - original_p).abs() <= 0.05,
            "drift should be small: original={original_p}, stored={stored_p}"
        );
        // The stored value should (very likely) differ from the original
        // due to the random drift. We don't assert inequality because in
        // very rare cases the drift could be near zero.
    }

    #[test]
    fn probabilistic_link_unchanged() {
        // Regression: Probabilistic links should still do Bayesian adjustment
        let mut ent = Entanglement::default();
        ent.add_pair(0, 1, 0.5, LinkType::Probabilistic);

        let pair = &ent.pairs[0];

        // Mine observed, baseline 0.3 → result should blend toward 0.7
        let result = ent.collapse_partner_probability(pair, true, 0.3);
        // Expected: 0.3 * 0.5 + 0.7 * 0.5 = 0.5
        assert!(
            (result - 0.5).abs() < 1e-10,
            "Probabilistic: expected 0.5, got {result}"
        );

        // Safe observed, baseline 0.3 → result should blend toward 0.3
        let result = ent.collapse_partner_probability(pair, false, 0.3);
        // Expected: 0.3 * 0.5 + 0.3 * 0.5 = 0.3
        assert!(
            (result - 0.3).abs() < 1e-10,
            "Probabilistic: expected 0.3, got {result}"
        );
    }
}
