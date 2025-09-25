# Bittensor Subnet Registration Bot

`regbot` is a Rust-based automated registration bot designed for the Bittensor blockchain network. It streamlines the process of registering hotkeys using user-provided coldkeys and other customizable parameters. The bot interacts directly with both the blockchain and subtensor endpoints, making it suitable for automated, large-scale, or repeated registration tasks.

This project is intended for developers, node operators, and researchers who need to programmatically register accounts (hotkeys) on Bittensor, monitor costs, and handle blockchain events efficiently.

## Features

- **Automated Hotkey Registration:** Easily register hotkeys on the Bittensor blockchain using your coldkey and network parameters.
- **Flexible Configuration:** Configure all parameters via command line flags or a TOML config file for repeatable deployments.
- **Batch Transaction Support:** Handles batch transactions and parses blockchain events for robust feedback and error handling.
- **Cost Monitoring:** Checks and respects recycle cost thresholds to avoid expensive transactions.
- **Rate Limiting:** Implements rate limiting to avoid spamming the network and to comply with best practices.
- **Comprehensive Logging:** Uses the Rust `log` and `env_logger` crates for detailed runtime information, error reporting, and debugging.

## Usage

### Prerequisites

### Prerequisites

- **Rust (edition 2021):** Install Rust from [rustup.rs](https://rustup.rs/).
- **Blockchain Access:** You need access to a Substrate-compatible blockchain endpoint (default provided).
- **Keypairs:** Valid coldkey and hotkey (SR25519 keypairs, can be seed phrases or URIs).

### Build Instructions

Clone the repository and build the project in release mode:

```powershell
git clone <repo-url>
cd regbot
cargo build --release
```

### Running the Bot

You can run the bot with command line arguments. All parameters are required unless a default is specified.

```powershell
cargo run --release -- \
    --coldkey "<COLDKEY mnemonic word>" \
    --hotkey "<HOTKEY mnemonic word>" \
    --netuid <NETUID> \
    [--max-cost <MAX_COST> (optional)] \
```
or after build
```
./regbot --coldkey "<COLDKEY mnemonic word>" --hotkey "<HOTKEY mnemonic word>" --netuid <NETUID>
```

#### Example Usage

```powershell
cargo run --release -- --coldkey "garden cherry orbit fabric loyal drift wisdom ocean cactus enrich drama shell" --hotkey "orange vivid canyon crisp umbrella talent eagle fossil shrimp velvet adapt breeze" --netuid 1 [--max-cost 6000000000]
```

### Parameters Explained

- `--coldkey`: Coldkey seed phrase or URI (used for signing and authentication)
- `--hotkey`: Hotkey seed phrase or URI (the account to be registered)
- `--netuid`: Network UID (u16, identifies the subnet/network)
- `--max-cost`: Maximum allowed recycle cost (default: 5000000000, prevents expensive transactions)
- `--chain-endpoint`: WebSocket endpoint for blockchain (default: `wss://entrypoint-finney.opentensor.ai:443`)
- `--seed`: Optional seed value for deterministic key generation

### Output and Logging

The bot provides detailed logs for every step, including cost checks, transaction attempts, event parsing, and errors. Logs are printed to the console and can be filtered by log level (default: INFO).

Successful registration will be confirmed in the logs, along with block and event details.
- [chrono](https://crates.io/crates/chrono)
- [chrono-tz](https://crates.io/crates/chrono-tz)
- [parity-scale-codec](https://crates.io/crates/parity-scale-codec)

See `Cargo.toml` for the full list and versions.

## License
MIT

## Author
See `Cargo.toml` for authorship info.

---

## Project Background & Motivation

Bittensor is a decentralized, blockchain-based protocol for machine intelligence. Registering hotkeys is a fundamental step for participating in the network, whether for mining, staking, or running validators. Manual registration can be error-prone and slow, especially for large-scale operations. `regbot` automates this process, providing reliability, repeatability, and transparency for node operators and developers.

For questions, issues, or contributions, please open an issue or pull request on the repository.
