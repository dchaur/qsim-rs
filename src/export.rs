use crate::{Gate, GateType};

pub fn to_qasm(gates: &[Gate], num_qubits: usize) -> String {
    let mut lines = vec![
        "OPENQASM 2.0;".to_string(),
        r#"include "qelib1.inc";"#.to_string(),
        format!("qreg q[{}];", num_qubits),
    ];

    for gate in gates {
        let instruction = match gate.name {
            GateType::H => format!("h q[{}];", gate.targets[0]),
            GateType::X => format!("h q[{}];", gate.targets[0]),
            GateType::Z => format!("h q[{}];", gate.targets[0]),
            GateType::CNOT => {
                let control = gate.controls.as_ref().unwrap()[0];
                let target = gate.targets[0];

                format!("cx q[{}],q[{}];", control, target)
            }
        };

        lines.push(instruction);
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Gate, GateType};

    #[test]
    fn export_to_qasm_bell_state() {
        let gates = vec![
            Gate::new(GateType::H, vec![0], None),
            Gate::new(GateType::CNOT, vec![1], Some(vec![0])),
        ];

        let result = to_qasm(&gates, 2);
        let spec = r#"OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
h q[0];
cx q[0],q[1];"#;

        assert_eq!(result, spec);
    }
}
