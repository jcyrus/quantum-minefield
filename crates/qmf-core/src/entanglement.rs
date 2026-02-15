use serde::{Deserialize, Serialize};

/// The type of quantum link between two entangled cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// Bayesian probability adjustment — weak correlation.
    Probabilistic,
    /// True Bell State — perfect anti-correlation. Observing one
    /// instantly collapses the partner to a definite state.
    BellState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntanglementPair {
    pub left: usize,
    pub right: usize,
    pub strength: f64,
    pub link_type: LinkType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entanglement {
    pub pairs: Vec<EntanglementPair>,
}

impl Entanglement {
    pub fn add_pair(&mut self, left: usize, right: usize, strength: f64, link_type: LinkType) {
        self.pairs.push(EntanglementPair {
            left,
            right,
            strength: strength.clamp(0.0, 1.0),
            link_type,
        });
    }

    /// Find the **first** partner for a given cell index.
    pub fn partner_of(&self, index: usize) -> Option<(&EntanglementPair, usize)> {
        self.pairs.iter().find_map(|pair| {
            if pair.left == index {
                Some((pair, pair.right))
            } else if pair.right == index {
                Some((pair, pair.left))
            } else {
                None
            }
        })
    }

    /// Find **all** partners for a given cell index (needed for GHZ chains).
    pub fn partners_of(&self, index: usize) -> Vec<(&EntanglementPair, usize)> {
        self.pairs
            .iter()
            .filter_map(|pair| {
                if pair.left == index {
                    Some((pair, pair.right))
                } else if pair.right == index {
                    Some((pair, pair.left))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Compute the partner's new probability after observing a cell.
    ///
    /// - **`BellState`**: Perfect anti-correlation. If a mine was observed the
    ///   partner is safe (`0.0`); if safe was observed the partner is a mine
    ///   (`1.0`).
    /// - **`Probabilistic`**: Bayesian blend weighted by `strength`.
    pub fn collapse_partner_probability(
        &self,
        pair: &EntanglementPair,
        observed_mine: bool,
        baseline_probability: f64,
    ) -> f64 {
        match pair.link_type {
            LinkType::BellState => {
                // Perfect anti-correlation: partner is the opposite.
                if observed_mine {
                    0.0 // partner is definitely safe
                } else {
                    1.0 // partner is definitely a mine
                }
            }
            LinkType::Probabilistic => {
                let baseline = baseline_probability.clamp(0.0, 1.0);
                let entangled_target = if observed_mine {
                    1.0 - baseline
                } else {
                    baseline
                };
                (baseline * (1.0 - pair.strength) + entangled_target * pair.strength)
                    .clamp(0.0, 1.0)
            }
        }
    }
}
