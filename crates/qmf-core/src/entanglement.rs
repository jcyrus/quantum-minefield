use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntanglementPair {
    pub left: usize,
    pub right: usize,
    pub strength: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entanglement {
    pub pairs: Vec<EntanglementPair>,
}

impl Entanglement {
    pub fn add_pair(&mut self, left: usize, right: usize, strength: f64) {
        self.pairs.push(EntanglementPair {
            left,
            right,
            strength: strength.clamp(0.0, 1.0),
        });
    }

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

    pub fn collapse_partner_probability(
        &self,
        pair: &EntanglementPair,
        observed_mine: bool,
        baseline_probability: f64,
    ) -> f64 {
        let baseline = baseline_probability.clamp(0.0, 1.0);
        let entangled_target = if observed_mine {
            1.0 - baseline
        } else {
            baseline
        };
        (baseline * (1.0 - pair.strength) + entangled_target * pair.strength).clamp(0.0, 1.0)
    }
}
