use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gate {
    Hadamard,
    Not,
    PhaseShift(f64),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Circuit {
    pub gates: Vec<Gate>,
}

impl Circuit {
    pub fn with_gate(mut self, gate: Gate) -> Self {
        self.gates.push(gate);
        self
    }

    /// Apply the gate chain to an input probability, producing a scrambled
    /// output in \[0, 1\]. This is the player-visible "hint" probability —
    /// higher circuit complexity makes the hints less reliable.
    pub fn apply_probability(&self, input: f64) -> f64 {
        self.gates.iter().fold(input.clamp(0.0, 1.0), |p, gate| {
            match gate {
                // Hadamard: compress probability toward 0.5 by halving
                // distance from center.  H(0.2) = 0.35, H(0.8) = 0.65, H(0.5) = 0.5
                Gate::Hadamard => 0.5 + (p - 0.5) * 0.5,
                // Not: flip probability
                Gate::Not => 1.0 - p,
                // PhaseShift(θ): rotate probability using cos²/sin² mixing.
                // θ=0 → identity, θ=π → full flip.
                Gate::PhaseShift(theta) => {
                    let c2 = (theta / 2.0).cos().powi(2);
                    let s2 = (theta / 2.0).sin().powi(2);
                    (p * c2 + (1.0 - p) * s2).clamp(0.0, 1.0)
                }
            }
        })
    }

    /// Construct a difficulty-appropriate gate pipeline.
    ///
    /// - `"observer"`:   mild distortion — probabilities stay close to truth
    /// - `"researcher"`: moderate scrambling
    /// - `"theorist"`:   heavy scrambling — hints are unreliable
    pub fn for_difficulty(label: &str) -> Self {
        match label {
            "observer" => Self::default().with_gate(Gate::PhaseShift(std::f64::consts::FRAC_PI_6)),
            "theorist" => Self::default()
                .with_gate(Gate::Hadamard)
                .with_gate(Gate::PhaseShift(std::f64::consts::FRAC_PI_3))
                .with_gate(Gate::Hadamard),
            // "researcher" or any other label
            _ => Self::default()
                .with_gate(Gate::Hadamard)
                .with_gate(Gate::PhaseShift(std::f64::consts::FRAC_PI_4)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hadamard_compresses_toward_half() {
        let c = Circuit::default().with_gate(Gate::Hadamard);
        assert!((c.apply_probability(0.2) - 0.35).abs() < 1e-10);
        assert!((c.apply_probability(0.8) - 0.65).abs() < 1e-10);
        assert!((c.apply_probability(0.5) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn not_flips() {
        let c = Circuit::default().with_gate(Gate::Not);
        assert!((c.apply_probability(0.3) - 0.7).abs() < 1e-10);
    }

    #[test]
    fn phase_shift_zero_is_identity() {
        let c = Circuit::default().with_gate(Gate::PhaseShift(0.0));
        assert!((c.apply_probability(0.3) - 0.3).abs() < 1e-10);
        assert!((c.apply_probability(0.7) - 0.7).abs() < 1e-10);
    }

    #[test]
    fn phase_shift_pi_is_flip() {
        let c = Circuit::default().with_gate(Gate::PhaseShift(std::f64::consts::PI));
        // cos²(π/2) = 0, sin²(π/2) = 1 → output = (1-p)
        assert!((c.apply_probability(0.3) - 0.7).abs() < 1e-10);
    }

    #[test]
    fn difficulty_pipelines_differ() {
        let obs = Circuit::for_difficulty("observer").apply_probability(0.15);
        let res = Circuit::for_difficulty("researcher").apply_probability(0.15);
        let the = Circuit::for_difficulty("theorist").apply_probability(0.15);
        // All should produce outputs in [0, 1]
        for v in [obs, res, the] {
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
        // Observer should stay closest to input
        assert!((obs - 0.15).abs() < (res - 0.15).abs());
    }
}
