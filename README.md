# MORALE - AI Skills Auditor

MORALE is a Rust-based tool designed to audit AI skills for various security risks including supply chain vulnerabilities, network risks, context poisoning, and prompt injection attacks. It operates as a single binary and uses an embedded SurrealDB database to store audit results.

## Features

- **Multi-risk Assessment**: Checks for supply chain, network, context poisoning, and prompt injection risks
- **Embedded Database**: Uses SurrealDB with memory storage for fast, local data persistence
- **Flexible Input**: Can audit both uncompressed folders and ZIP files containing AI skills
- **Detailed Reporting**: Provides comprehensive risk analysis with evidence and severity ratings
- **Modular Architecture**: Clean, extensible codebase organized into distinct modules

## Installation

### Prerequisites

- Rust 1.70+ 
- Cargo

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-org/morale.git
cd morale

# Build the project
cargo build --release

# Run the auditor
./target/release/morale <path-to-ai-skill>
```

## Usage

### Basic Usage

```bash
# Audit a single AI skill
morale /path/to/ai-skill-folder

# Audit a ZIP file containing an AI skill
morale /path/to/ai-skill.zip

# Verbose output
morale -v /path/to/ai-skill

# Lightweight run (Skip database saving/operations and print findings directly to the terminal)
morale --lightweight /path/to/ai-skill
# or using the short flag:
morale -l /path/to/ai-skill
```

### Configuration

MORALE uses a TOML configuration file (`config.toml`) that allows customization of:

- Database settings
- Risk thresholds
- Enabled checks
- Report formats
- Security limits

Example configuration:

```toml
[database]
storage_engine = "mem"
namespace = "morale"
database = "skills"

[audit]
verbose = false
enabled_checks = [
    "supply_chain",
    "network", 
    "context_poisoning",
    "prompt_injection"
]

[report]
output_format = "json"
output_directory = "./reports"
include_evidence = true
```

## Architecture

MORALE follows a modular architecture with the following components:

- **Models**: Defines data structures for AI skills and risk findings
- **Risk Checkers**: Implements specific checks for different risk types
- **Database**: Handles storage and retrieval of audit results using SurrealDB
- **Main**: Orchestrates the auditing process

## Risk Categories

MORALE checks for the following risk categories:

1. **Supply Chain Risks**: Vulnerabilities related to dependencies and source trustworthiness
2. **Network Risks**: Issues with network communications and data transmission
3. **Context Poisoning**: Risks related to state management and memory retention
4. **Prompt Injection**: Vulnerabilities in input handling and prompt processing

## Contributing

We welcome contributions! Please see our contributing guidelines for more information.

## License

This project is licensed under the MIT License - see the LICENSE file for details.