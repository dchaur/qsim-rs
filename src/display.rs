use num_complex::Complex64;
use thiserror::Error;

use crate::{Gate, GateType};

#[derive(Debug, PartialEq, Error)]
pub enum MeasurementError {
    #[error("cannot format measurement: no qubits provided")]
    EmptyBits,
}

enum CellSymbol {
    Gate(GateType),
    Control,
    Target,
    Wire,
}

enum RowSymbol {
    Connector,
    Empty,
}

fn index_to_label(index: usize, num_qubits: usize) -> String {
    (0..num_qubits)
        .rev()
        .map(|q| if ((index >> q) & 1) == 1 { '1' } else { '0' })
        .collect()
}

fn dirac_label(binary: &str, index: usize) -> String {
    format!("|{}⟩ ({})", binary, index)
}

pub fn format_measurement(bits: &[u8]) -> Result<String, MeasurementError> {
    (!bits.is_empty())
        .then_some(())
        .ok_or(MeasurementError::EmptyBits)?;

    let decimal = bits.iter().fold(0u64, |acc, &bit| (acc << 1) | bit as u64);
    let label = dirac_label(
        &index_to_label(decimal as usize, bits.len()),
        decimal as usize,
    );
    Ok(format!("Measured: {}", label))
}

pub fn format_probabilities(state: &[Complex64], num_qubits: usize) -> String {
    state
        .iter()
        .enumerate()
        .map(|(index, &amplitude)| {
            let percentage = amplitude.norm_sqr() * 100.0;
            let label = dirac_label(&index_to_label(index, num_qubits), index);

            format!("  {}  →  {:.2}%", label, percentage)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn cell_to_str(symbol: &CellSymbol) -> &str {
    match symbol {
        CellSymbol::Wire => "─────",
        CellSymbol::Control => "──●──",
        CellSymbol::Target => "──X──",
        CellSymbol::Gate(GateType::X) => "──X──",
        CellSymbol::Gate(GateType::H) => "──H──",
        CellSymbol::Gate(GateType::Z) => "──Z──",
        CellSymbol::Gate(GateType::CNOT) => "─────",
    }
}

fn row_to_str(symbol: &RowSymbol) -> &str {
    match symbol {
        RowSymbol::Connector => "  │  ",
        RowSymbol::Empty => "     ",
    }
}

fn is_control(gate: &Gate, qubit: usize) -> bool {
    gate.controls.as_ref().and_then(|v| v.first()).copied() == Some(qubit)
}

fn cnot_spans_row(gate: &Gate, between_row: usize) -> bool {
    let control = gate.controls.as_ref().and_then(|v| v.first().copied());
    let target = gate.targets.first().copied();

    let (Some(c), Some(t)) = (control, target) else {
        return false;
    };

    let min = c.min(t);
    let max = c.max(t);

    between_row >= min && between_row < max
}

pub fn format_circuit(gates: &[Gate], num_qubits: usize) -> String {
    let mut final_rows = Vec::new();

    for q in 0..num_qubits {
        let cell_symbols: Vec<CellSymbol> = gates
            .iter()
            .map(|gate| match gate.name {
                GateType::X | GateType::H | GateType::Z if gate.targets[0] == q => {
                    CellSymbol::Gate(gate.name.clone())
                }
                GateType::CNOT if is_control(gate, q) => CellSymbol::Control,
                GateType::CNOT if gate.targets.first() == Some(&q) => CellSymbol::Target,
                _ => CellSymbol::Wire,
            })
            .collect();

        let row: String = cell_symbols
            .iter()
            .map(|symbol| cell_to_str(symbol))
            .collect();

        final_rows.push(format!("q{}: {}", q, row));

        if q < num_qubits - 1 {
            let spacer_row: String = gates
                .iter()
                .map(|gate| {
                    if gate.name == GateType::CNOT && cnot_spans_row(gate, q) {
                        row_to_str(&RowSymbol::Connector)
                    } else {
                        row_to_str(&RowSymbol::Empty)
                    }
                })
                .collect();
            final_rows.push(format!("    {}", spacer_row));
        }
    }

    final_rows.join("\n")
}

#[cfg(test)]
mod tests {
    use crate::Gate;

    use super::*;
    use std::f64::consts::FRAC_1_SQRT_2;

    #[test]
    fn index_to_label_pads_correctly() {
        assert_eq!(index_to_label(0, 2), "00");
        assert_eq!(index_to_label(1, 2), "01");
        assert_eq!(index_to_label(3, 2), "11");
        assert_eq!(index_to_label(5, 3), "101");
    }

    #[test]
    fn converts_binary_string_into_dirac_notation() {
        assert_eq!(dirac_label("00", 0), "|00⟩ (0)");
        assert_eq!(dirac_label("01", 1), "|01⟩ (1)");
        assert_eq!(dirac_label("11", 3), "|11⟩ (3)");
        assert_eq!(dirac_label("101", 5), "|101⟩ (5)");
    }

    #[test]
    fn format_one_qubit_circuit() {
        assert_eq!(format_measurement(&[1]), Ok("Measured: |1⟩ (1)".into()));
    }

    #[test]
    fn format_three_qubit_circuit() {
        assert_eq!(
            format_measurement(&[1, 0, 1]),
            Ok("Measured: |101⟩ (5)".into())
        );
    }

    #[test]
    fn format_all_zeros() {
        assert_eq!(format_measurement(&[0, 0]), Ok("Measured: |00⟩ (0)".into()));
    }

    #[test]
    fn format_empty_circuit() {
        assert_eq!(format_measurement(&[]), Err(MeasurementError::EmptyBits));
    }

    #[test]
    fn format_one_qubit_probabilities() {
        let probs = format_probabilities(&[Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)], 1);
        assert_eq!(probs, "  |0⟩ (0)  →  100.00%\n  |1⟩ (1)  →  0.00%");
    }

    #[test]
    fn format_bell_state() {
        let bell_state = vec![
            Complex64::new(FRAC_1_SQRT_2, 0.0), // |00⟩ → 50%
            Complex64::new(0.0, 0.0),           // |01⟩ →  0%
            Complex64::new(0.0, 0.0),           // |10⟩ →  0%
            Complex64::new(FRAC_1_SQRT_2, 0.0), // |11⟩ → 50%
        ];
        let probs = format_probabilities(&bell_state, 2);

        assert_eq!(
            probs,
            "  |00⟩ (0)  →  50.00%\n  |01⟩ (1)  →  0.00%\n  |10⟩ (2)  →  0.00%\n  |11⟩ (3)  →  50.00%"
        );
    }

    #[test]
    fn cell_to_string_correctly_translated() {
        assert_eq!(cell_to_str(&CellSymbol::Wire), "─────");
        assert_eq!(cell_to_str(&CellSymbol::Control), "──●──");
        assert_eq!(cell_to_str(&CellSymbol::Target), "──X──");
        assert_eq!(cell_to_str(&CellSymbol::Gate(GateType::X)), "──X──");
        assert_eq!(cell_to_str(&CellSymbol::Gate(GateType::H)), "──H──");
        assert_eq!(cell_to_str(&CellSymbol::Gate(GateType::Z)), "──Z──");
        assert_eq!(cell_to_str(&CellSymbol::Gate(GateType::CNOT)), "─────");
    }

    #[test]
    fn row_to_string_correctly_translated() {
        assert_eq!(row_to_str(&RowSymbol::Connector), "  │  ");
        assert_eq!(row_to_str(&RowSymbol::Empty), "     ");
    }

    #[test]
    fn check_cnot_does_not_span_row() {
        // Spacer after the span
        let g1 = Gate::new(GateType::CNOT, vec![1], Some(vec![0]));
        assert_eq!(cnot_spans_row(&g1, 1), false);

        // Spacer before the span
        let g2 = Gate::new(GateType::CNOT, vec![2], Some(vec![1]));
        assert_eq!(cnot_spans_row(&g2, 0), false);
    }
    #[test]
    fn format_circuit_x_gate() {
        let gates = vec![Gate::new(crate::GateType::X, vec![0], None)];
        let result = format_circuit(&gates, 1);

        assert_eq!(result, "q0: ──X──");
    }

    #[test]
    fn format_circuit_bell_state_with_spacers() {
        let gates = vec![
            Gate::new(GateType::H, vec![0], None),
            Gate::new(GateType::CNOT, vec![1], Some(vec![0])),
        ];
        let result = format_circuit(&gates, 2);

        let expected = concat!("q0: ──H────●──\n", "           │  \n", "q1: ───────X──");

        assert_eq!(result, expected);
    }
}
