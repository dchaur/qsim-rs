use std::fs;

use clap::{Parser, Subcommand};

use quantum_circuit_cli::{
    Circuit, Gate, GateType, OracleType, deutsch_jozsa, format_circuit, format_measurement,
    format_probabilities, load_circuit, oracle_balanced, oracle_constant_false, save_circuit,
    to_qasm,
};

#[derive(Parser)]
#[command(name = "qsim", version, about = "Quantum Circuit Simulator CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(short, long, default_value = "Quantum Circuit")]
        name: String,

        #[arg(short, long, default_value_t = 2)]
        qubits: usize,
    },
    AddGate {
        #[arg(value_enum)]
        gate: GateType,

        #[arg(num_args = 1..=2, required = true)]
        targets: Vec<usize>,
    },
    Load {
        #[arg(long)]
        path: String,
    },
    Demo {
        #[arg(long)]
        save: Option<String>,
    },
    Export {
        #[arg(long)]
        from: String,

        #[arg(long)]
        to: String,
    },
    DeutschJozsa {
        #[arg(long)]
        oracle: OracleType,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Run { name, qubits } => {
            println!("Initializing circuit: '{}' with {} qubits.", name, qubits);
            let mut _circuit = Circuit::new(name, qubits);
        }
        Commands::AddGate { gate, targets } => {
            println!("Gate detected by CLI: {:?} on qubits {:?}", gate, targets);

            // Create the gate using the constructor from domain.rs
            // Note: For CNOT you could separate controls from targets later
            let _new_gate = Gate::new(gate, targets, None);

            println!("Gate validated and constructed successfully!");
        }
        Commands::Demo { save } => {
            let demo_name = "Bell State Demo";
            let num_of_qubits = 2;
            let mut circuit = Circuit::new(demo_name.into(), num_of_qubits);
            circuit.add_gate(Gate::new(GateType::H, vec![0], None));
            circuit.add_gate(Gate::new(GateType::CNOT, vec![1], Some(vec![0])));

            circuit.execute();

            println!("{} ({} qubits)", demo_name, num_of_qubits);
            println!("{}", format_circuit(&circuit.gates, num_of_qubits));
            println!("{}", format_probabilities(&circuit.state, num_of_qubits));

            let result = circuit.measure_all();

            match format_measurement(&result) {
                Ok(result) => println!("{}", result),
                Err(e) => eprintln!("Error: {}", e),
            }
            println!("---");

            if let Some(path) = save {
                match save_circuit(&circuit, &path.to_string()) {
                    Ok(_) => println!(
                        "The circuit {} has been successfully saved in {}",
                        circuit.name, path
                    ),
                    Err(e) => eprintln!("There was a problem saving the circuit:  {}", e),
                }
            }
        }
        Commands::Load { path } => match load_circuit(&path) {
            Ok(circuit) => {
                println!("{} ({} qubits)", circuit.name, circuit.num_qubits);
                println!("{}", format_circuit(&circuit.gates, circuit.num_qubits));
                println!(
                    "{}",
                    format_probabilities(&circuit.state, circuit.num_qubits)
                );
            }
            Err(e) => eprintln!("Error:  {}", e),
        },
        Commands::Export { from, to } => match load_circuit(&from) {
            Ok(circuit) => {
                println!("{} ({} qubits)", circuit.name, circuit.num_qubits);

                let qasm_result = to_qasm(&circuit.gates, circuit.num_qubits);
                if let Err(e) = fs::write(to, qasm_result) {
                    eprintln!("Error writing file: {}", e)
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::DeutschJozsa { oracle } => {
            let num_of_qubits = 2;
            let oracle_fn = match oracle {
                OracleType::Constant => oracle_constant_false,
                OracleType::Balanced => oracle_balanced,
            };

            let (circuit, is_balanced) = deutsch_jozsa(oracle_fn);
            println!("{} ({} qubits)", "Deutsch's algorithm", num_of_qubits);
            println!("");
            println!("{}", format_circuit(&circuit.gates, num_of_qubits));
            println!("");
            println!("Result: {}", if is_balanced { "BALANCED" } else { "CONSTANT" });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cli_command() {
        let res = Cli::try_parse_from(["qsim", "add-gate", "x", "0"]);

        assert!(res.is_ok());
    }

    #[test]
    fn test_invalid_gate_command() {
        let res = Cli::try_parse_from(["qsim", "add-gate", "not-a-gate", "x", "0"]);

        assert!(res.is_err());
    }

    #[test]
    fn should_export_a_file_to_qasm() {
        let res =
            Cli::try_parse_from(["qsim", "export", "--from", "bell.json", "--to", "bell.qasm"]);

        assert!(res.is_ok());
    }
}
