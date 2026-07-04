use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Circuit, Gate};

#[derive(Debug, Serialize, Deserialize)]
struct CircuitFile {
    pub name: String,
    pub num_qubits: usize,
    pub gates: Vec<Gate>,
}

impl From<&Circuit> for CircuitFile {
    fn from(circuit: &Circuit) -> Self {
        CircuitFile {
            name: circuit.name.clone(),
            num_qubits: circuit.num_qubits,
            gates: circuit.gates.clone(),
        }
    }
}

pub fn save_circuit(circuit: &Circuit, path: &str) -> Result<()> {
    let contents = serde_json::to_string_pretty(&CircuitFile::from(circuit))?;
    fs::write(path, contents)?;
    Ok(())
}

pub fn load_circuit(path: &str) -> Result<Circuit> {
    let content = fs::read_to_string(path)?;
    let file: CircuitFile = serde_json::from_str(&content)?;

    let mut circuit = Circuit::new(file.name, file.num_qubits);

    for gate in file.gates {
        circuit.add_gate(gate);
    }

    circuit.execute();
    Ok(circuit)
}

#[cfg(test)]
mod tests {
    use crate::GateType;

    use super::*;

    #[test]
    fn circuit_file_round_trips_to_json() {
        let fake_name = "Higgs boson";
        let num_qubits = 1;
        let mut circuit = Circuit::new(fake_name.to_string(), num_qubits);
        let g = Gate::new(GateType::X, vec![0], None);
        circuit.add_gate(g);

        let original = CircuitFile::from(&circuit);

        let json = serde_json::to_string(&original).unwrap();
        let restored: CircuitFile = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.name, fake_name);
        assert_eq!(restored.num_qubits, num_qubits);
        assert_eq!(restored.gates.len(), 1);
    }

    #[test]
    fn saves_the_circuit() {
        let path = "test_circuit.json";
        let mut circuit = Circuit::new("Higgs boson".to_string(), 1);
        circuit.add_gate(Gate::new(GateType::X, vec![0], None));

        save_circuit(&circuit, path).unwrap();

        let loaded = load_circuit(path).unwrap();
        assert_eq!(loaded.name, "Higgs boson");
        assert_eq!(loaded.gates.len(), 1);

        std::fs::remove_file(path).unwrap();
    }
}
