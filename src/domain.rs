use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f64::consts::FRAC_1_SQRT_2;

use clap::ValueEnum;
use num_complex::Complex64;

#[derive(Debug)]
pub struct Qubit {
    pub index: usize,
    pub state: [Complex64; 2], // |0> and |1> amplitudes
}

impl Qubit {
    pub fn new(index: usize) -> Qubit {
        Qubit {
            index,
            state: [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)], // Start in |0>
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum GateType {
    X,
    H,
    Z,
    CNOT,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OracleType {
    Constant,
    Balanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    pub name: GateType,
    pub targets: Vec<usize>,
    pub controls: Option<Vec<usize>>,
    // pub matrix: Option<ndarray::Array2<Complex64>> // For future expansion
}

impl Gate {
    pub fn new(name: GateType, targets: Vec<usize>, controls: Option<Vec<usize>>) -> Gate {
        Gate {
            name,
            targets,
            controls,
        }
    }
}

#[derive(Debug)]
pub struct Circuit {
    pub name: String,
    pub num_qubits: usize,
    pub state: Vec<Complex64>,
    pub gates: Vec<Gate>,
}

impl Circuit {
    pub fn new(name: String, num_qubits: usize) -> Circuit {
        let size = 1 << num_qubits;
        let mut state = vec![Complex64::new(0.0, 0.0); size];

        state[0] = Complex64::new(1.0, 0.0);

        Circuit {
            name,
            num_qubits,
            state,
            gates: Vec::new(),
        }
    }

    pub fn apply_x(&mut self, target: usize) {
        self.for_each_pair(target, |state, idx0, idx1| {
            state.swap(idx0, idx1);
        });
    }

    pub fn apply_h(&mut self, target: usize) {
        self.for_each_pair(target, |state, idx0, idx1| {
            let alpha = state[idx0];
            let beta = state[idx1];

            state[idx0] = (alpha + beta) * FRAC_1_SQRT_2;
            state[idx1] = (alpha - beta) * FRAC_1_SQRT_2;
        });
    }

    pub fn apply_z(&mut self, target: usize) {
        self.for_each_pair(target, |state, _idx0, idx1| {
            state[idx1] = -state[idx1];
        });
    }

    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        self.for_each_pair(target, |state, idx0, idx1| {
            if (idx0 >> control) & 1 == 1 {
                state.swap(idx0, idx1);
            }
        });
    }

    pub fn add_gate(&mut self, gate: Gate) {
        self.gates.push(gate);
    }

    pub fn execute(&mut self) {
        let gates = self.gates.clone();

        for gate in &gates {
            match gate.name {
                GateType::X => self.apply_x(gate.targets[0]),
                GateType::H => self.apply_h(gate.targets[0]),
                GateType::Z => self.apply_z(gate.targets[0]),
                GateType::CNOT => {
                    if let Some(ref controls) = gate.controls {
                        if !controls.is_empty() {
                            self.apply_cnot(controls[0], gate.targets[0]);
                        }
                    }
                }
            }
        }
    }

    pub fn measure_all(&mut self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen_range(0.0..1.0);

        // Step 1: Calculate individual probabilities
        let probabilities: Vec<f64> = self.state.iter().map(|amp| amp.norm_sqr()).collect();

        // Step 2: Cumulative Probability Selection (Roulette Wheel)
        let mut cumulative_prob = 0.0;
        let mut measured_index = 0;

        for (index, prob) in probabilities.iter().enumerate() {
            cumulative_prob += prob;
            if random_value <= cumulative_prob {
                measured_index = index;
                break;
            }
        }

        // Step 3: Collapse the entire global state vector
        self.state.fill(Complex64::new(0.0, 0.0));
        self.state[measured_index] = Complex64::new(1.0, 0.0);

        // Step 4: Convert the winning index to a binary string representation
        // E.g., index 2 with 3 qubits -> [0, 1, 0]
        let mut binary_result = Vec::with_capacity(self.num_qubits);
        for q in (0..self.num_qubits).rev() {
            let bit = ((measured_index >> q) & 1) as u8;
            binary_result.push(bit);
        }

        binary_result
    }

    fn for_each_pair<F>(&mut self, target: usize, mut op: F)
    where
        F: FnMut(&mut [Complex64], usize, usize),
    {
        let bit = 1 << target;
        let half_size = self.state.len() >> 1;

        for i in 0..half_size {
            let low = i & (bit - 1);
            let high = (i >> target) << (target + 1);
            let idx0 = high | low;
            let idx1 = idx0 | bit;

            op(&mut self.state, idx0, idx1);
        }
    }
}

pub fn oracle_constant_false(_x: bool) -> bool {
    false
}

pub fn oracle_balanced(x: bool) -> bool {
    x
}

pub fn deutsch_jozsa(oracle: fn(bool) -> bool) -> (Circuit, bool) {
    let mut circuit = Circuit::new("deutsch_jozsa".to_string(), 2);

    circuit.add_gate(Gate::new(GateType::H, vec![0], None));
    circuit.add_gate(Gate::new(GateType::X, vec![1], None));
    circuit.add_gate(Gate::new(GateType::H, vec![1], None));

    let flips = oracle(false) != oracle(true);

    if flips {
        circuit.add_gate(Gate::new(GateType::CNOT, vec![1], Some(vec![0])));
    }

    circuit.add_gate(Gate::new(GateType::H, vec![0], None));

    circuit.execute();

    let res = circuit.measure_all();

    (circuit, res[1] == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_constant_true(_x: bool) -> bool {
        true
    }

    #[test]
    fn can_create_qubit() {
        let q = Qubit::new(0);
        assert_eq!(q.index, 0);
        assert_eq!(
            q.state,
            [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)]
        );
    }

    #[test]
    fn can_create_gate() {
        let g = Gate::new(GateType::X, vec![0], None);
        matches!(g.name, GateType::X);
        assert_eq!(g.targets, vec![0]);
        assert!(g.controls.is_none());
    }

    #[test]
    fn can_create_circuit() {
        let c = Circuit::new("test_circuit".to_string(), 2);
        assert_eq!(c.name, "test_circuit");
        assert_eq!(c.num_qubits, 2);
        assert_eq!(c.state.len(), 4);

        assert_eq!(c.state[0], Complex64::new(1.0, 0.0));
        assert_eq!(c.state[1], Complex64::new(0.0, 0.0));

        assert!(c.gates.is_empty());
    }

    #[test]
    fn can_apply_gate_x() {
        let mut c = Circuit::new("test_circuit".to_string(), 2);

        c.apply_x(1);

        assert_eq!(c.state[0], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[1], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[2], Complex64::new(1.0, 0.0));
        assert_eq!(c.state[3], Complex64::new(0.0, 0.0));
    }

    #[test]
    fn can_apply_gate_x_triple_qubit() {
        let mut c = Circuit::new("test_circuit".to_string(), 3);

        c.apply_x(2);

        assert_eq!(c.state[0], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[1], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[2], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[3], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[4], Complex64::new(1.0, 0.0));
        assert_eq!(c.state[5], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[6], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[7], Complex64::new(0.0, 0.0));
    }

    #[test]
    fn can_apply_gate_x_single_qubit() {
        let mut c = Circuit::new("test_circuit".to_string(), 1);

        c.apply_x(0);

        assert_eq!(c.state[0], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[1], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn can_apply_gate_h_single_qubit() {
        let mut c = Circuit::new("test_circuit".to_string(), 1);

        c.apply_h(0);

        let expected = Complex64::new(FRAC_1_SQRT_2, 0.0);
        let epsilon = 1e-12;

        assert!(
            (c.state[0].re - expected.re).abs() < epsilon,
            "State [0] real out of range"
        );
        assert!(
            (c.state[0].im - expected.im).abs() < epsilon,
            "State [0] imaginary out of range"
        );

        assert!(
            (c.state[1].re - expected.re).abs() < epsilon,
            "State [1] real out of range"
        );
        assert!(
            (c.state[1].im - expected.im).abs() < epsilon,
            "State [1] imaginary out of range"
        );
    }

    #[test]
    fn apply_gate_cnot() {
        let mut c = Circuit::new("test_circuit".to_string(), 3);

        // 1. Set qubit 1 (control) to |1> using apply_x
        // The state goes from |000> (index 0) to |010> (index 2)
        c.apply_x(1);

        // 2. Apply CNOT: Control = Qubit 1, Target = Qubit 2
        // Since the control (qubit 1) is |1>, the target (qubit 2) MUST flip.
        // The circuit state should go from |010> (index 2) to |110> (index 6)
        c.apply_cnot(1, 2);

        // 3. Verify that the amplitude of 1.0 moved correctly to index 6
        assert_eq!(c.state[2], Complex64::new(0.0, 0.0)); // No longer at |010>
        assert_eq!(c.state[6], Complex64::new(1.0, 0.0)); // Now at |110>!
    }

    #[test]
    fn can_add_gate_to_circuit() {
        let mut c = Circuit::new("test".to_string(), 2);
        let gate = Gate::new(GateType::X, vec![0], None);

        c.add_gate(gate);

        assert_eq!(c.gates.len(), 1);
        assert!(matches!(c.gates[0].name, GateType::X));
    }

    #[test]
    fn can_execute_circuit_with_gate() {
        let mut c = Circuit::new("test".to_string(), 1);

        c.add_gate(Gate::new(GateType::X, vec![0], None));

        c.execute();

        assert_eq!(c.state[0], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[1], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn can_measure_qubit_in_superposition_statistical() {
        let mut count_zero = 0;
        let mut count_one = 0;

        for _ in 0..1000 {
            let mut c = Circuit::new("test".to_string(), 1);
            c.apply_h(0); // 50/50 superposition

            let result = c.measure_all();

            if result[0] == 0 {
                count_zero += 1;
            } else {
                count_one += 1;
            }
        }

        // Statistical check: should be ~500 each, +- tolerance
        assert!(count_zero > 450 && count_zero < 550);
        assert!(count_one > 450 && count_one < 550);
    }

    #[test]
    fn can_apply_gate_z_on_one() {
        let mut c = Circuit::new("test".to_string(), 1);

        c.apply_x(0);
        c.apply_z(0);

        assert_eq!(c.state[0], Complex64::new(0.0, 0.0));
        assert_eq!(c.state[1], Complex64::new(-1.0, 0.0));
    }

    #[test]
    fn can_apply_gate_z_on_superposition() {
        let mut c = Circuit::new("test_circuit".to_string(), 1);

        c.apply_h(0);
        c.apply_z(0);

        let epsilon = 1e-12;

        assert!(
            (c.state[0].re - FRAC_1_SQRT_2).abs() < epsilon,
            "State [0] should be +1/√2"
        );
        assert!(
            (c.state[1].re - (-FRAC_1_SQRT_2)).abs() < epsilon,
            "State [1] should be -1/√2"
        );
    }

    #[test]
    fn oracle_constant_false_returns_same_result() {
        assert_eq!(oracle_constant_false(true), oracle_constant_false(false));
    }

    #[test]
    fn oracle_constant_true_returns_same_result() {
        assert_eq!(oracle_constant_true(false), oracle_constant_true(true));
    }

    #[test]
    fn oracle_balanced_returns_based_on_input() {
        assert_ne!(oracle_balanced(false), oracle_balanced(true));
        assert_ne!(oracle_balanced(true), oracle_balanced(false));
    }

    #[test]
    fn deutsch_jozsa_oracle_constant_false() {
        let (_, res) = deutsch_jozsa(oracle_constant_false);

        assert_eq!(res, false);
    }

    #[test]
    fn deutsch_jozsa_oracle_constant_true() {
        let (_, res) = deutsch_jozsa(oracle_constant_true);

        assert_eq!(res, false);
    }

    #[test]
    fn deutsch_jozsa_oracle_balanced() {
        let (_, res) = deutsch_jozsa(oracle_balanced);

        assert_eq!(res, true);
    }
}
