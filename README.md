# Quantum Circuit Simulator CLI 🦀⚛️

A high-performance quantum circuit simulator built in Rust, featuring a clean CLI interface and efficient state vector implementation.

## ✨ Features

### Core Quantum Operations

- **Global State Vector**: Efficient $2^n$ state representation for n qubits
- **Quantum Gates**: X (NOT), H (Hadamard), Z (Phase flip), CNOT (Controlled-NOT)
- **Measurement**: Born rule implementation with wave function collapse
- **Circuit Execution**: Queue gates and execute in sequence

### Technical Highlights

- 🚀 **Optimized bit manipulation** for gate operations
- 🧪 **TDD approach** with 38+ unit tests covering all core functionality
- 🔧 **DRY architecture** with reusable `for_each_pair` helper
- 📊 **Statistical validation** (50/50 Hadamard distribution over 1000 measurements)

## 🛠️ Installation

### Prerequisites

- Rust 1.70+ (with Cargo)

### Build from Source

```bash
git clone <your-repo-url>
cd quantum-circuit-cli
cargo build --release
```

### Run Tests

```bash
cargo test
```

## 📖 Usage

### Basic Example: Quantum "Hello World"

```rust
use quantum_circuit_cli::Circuit;

// Create a 1-qubit circuit
let mut circuit = Circuit::new("hello_quantum".to_string(), 1);

// Apply Hadamard gate (creates superposition)
circuit.apply_h(0);

// Measure (collapses to |0⟩ or |1⟩ with 50% probability each)
let result = circuit.measure_all();
println!("Measured: {:?}", result); // [0] or [1]
```

### Multi-Qubit Example: Bell State

```rust
// Create a 2-qubit circuit
let mut circuit = Circuit::new("bell_state".to_string(), 2);

// Create entanglement
circuit.apply_h(0);           // Superposition on qubit 0
circuit.apply_cnot(0, 1);     // Entangle qubit 1 with qubit 0

// Measurement will always give [0,0] or [1,1] (never [0,1] or [1,0])
let result = circuit.measure_all();
```

### Using the Gate Queue

```rust
use quantum_circuit_cli::{Circuit, Gate, GateType};

let mut circuit = Circuit::new("queued".to_string(), 2);

// Add gates to queue
circuit.add_gate(Gate::new(GateType::H, vec![0], None));
circuit.add_gate(Gate::new(GateType::X, vec![1], None));
circuit.add_gate(Gate::new(GateType::CNOT, vec![1], Some(vec![0])));

// Execute all gates
circuit.execute();

// Measure
let result = circuit.measure_all();
```

## 🏗️ Architecture

### Module Structure

```
src/
├── lib.rs           # Public API
├── main.rs          # CLI entry point
├── domain.rs        # Core quantum logic
│   ├── Qubit        # Individual qubit type
│   ├── Gate         # Gate type + targets/controls
│   └── Circuit      # State vector + gate queue + operations
├── display.rs       # Output formatting
│   ├── format_measurement    # Dirac notation result
│   ├── format_probabilities  # State distribution before collapse
│   └── format_circuit        # ASCII circuit diagram
└── persistence.rs   # JSON save/load
    ├── save_circuit  # Serialize circuit to JSON file
    └── load_circuit  # Deserialize circuit from JSON file
```

### Key Design Decisions

1. **Global State Vector**: Instead of individual qubit states, we use a single vector of size $2^n$ to represent the entire quantum system. This enables proper entanglement simulation.

2. **Bit Manipulation**: Gates are implemented using efficient bit operations to find state pairs that need to be transformed.

3. **Measurement Strategy**: One random number selects the collapsed state (preserving quantum correlations), not one per qubit.

## 🧪 Testing

Current test coverage:

- ✅ Core type constructors
- ✅ Single-qubit gates (X, H, Z)
- ✅ Multi-qubit gates (CNOT)
- ✅ Gate queueing and execution
- ✅ Statistical measurement validation
- ✅ Dirac notation formatting
- ✅ State probability display
- ✅ ASCII circuit diagram with connectors
- ✅ JSON serialization/deserialization round-trip
- ✅ QASM export
- ✅ Deutsch-Jozsa algorithm (constant and balanced oracles)

Run tests with:

```bash
cargo test                     # All tests
cargo test --nocapture         # With output
cargo test measure             # Just measurement tests
```

## 📊 Completed Milestones

- ✅ **Milestone 1**: Project setup & core types
- ✅ **Milestone 2**: CLI parsing (clap integration)
- ✅ **Milestone 3**: Quantum core (gates + state vector)
- ✅ **Milestone 4**: Virtual machine (execution + measurement)
- ✅ **Milestone 5**: Reporter/UX module (display, diagram, probabilities)
- ✅ **Milestone 6**: Persistence (JSON save/load, `demo --save`, `load --path`)
- ✅ **Milestone 7**: Advanced features (QASM export, Deutsch-Jozsa algorithm)

## 🚧 Roadmap

### Future Ideas

- [ ] TUI visualization
- [ ] Grover's search algorithm
- [ ] Additional gates (Y, Phase, T)
- [ ] Multi-qubit QASM export

## 🤝 Contributing

This project is a learning exercise following TDD principles. Contributions, suggestions, and feedback are welcome!

## 📚 Learning Resources

If you're new to quantum computing:

- [Quantum Computing for the Very Curious](https://quantum.country/)
- [Qiskit Textbook](https://qiskit.org/textbook/)
- [Quantum Katas](https://github.com/microsoft/QuantumKatas)

## 📄 License

MIT License - see LICENSE file for details

---

**Built with ❤️ and Rust** - A journey into quantum computing fundamentals
