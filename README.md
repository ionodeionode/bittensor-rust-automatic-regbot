# Bittensor Subnet Automatic Registration Bot (`regbot`)

`regbot` is a Rust-based automated registration bot designed for the **Bittensor blockchain network**. It simplifies the process of registering hotkeys using user-provided coldkeys and customizable parameters. With robust features like cost monitoring, batch processing, and detailed logging, `regbot` is ideal for automated, large-scale, or repeated registration tasks.

## Key Features

- **Automated Hotkey Registration**: Register hotkeys on the Bittensor blockchain using your coldkey and network parameters.
- **Flexible Configuration**: Configure all parameters via command-line flags or a TOML configuration file for repeatable deployments.
- **Batch Transaction Support**: Handles batch transactions and parses blockchain events for robust feedback and error handling.
- **Cost Monitoring**: Avoid expensive transactions with configurable recycle cost thresholds.
- **Rate Limiting**: Prevent spamming the network and comply with best practices using built-in rate limiting.
- **Comprehensive Logging**: Detailed runtime logs with error reporting and debugging capabilities.

## Why Use `regbot`?

Bittensor is a decentralized, blockchain-based protocol for machine intelligence. Registering hotkeys is a critical step for participating in the network, whether for mining, staking, or running validators. Manual registration can be error-prone and slow, especially for large-scale operations. `regbot` automates this process, ensuring reliability, repeatability, and transparency for developers, node operators, and researchers.

## Prerequisites

- **Rust (edition 2021)**: Install Rust from [rustup.rs](https://rustup.rs/).
- **Blockchain Access**: Access to a Substrate-compatible blockchain endpoint (default provided).
- **Keypairs**: Valid coldkey and hotkey (SR25519 keypairs, can be seed phrases or URIs).

## Build Instructions

Clone the repository and build the project in release mode:

```bash
git clone <repo-url>
cd regbot
cargo build --release
```

## Running the Bot

Run the bot using command-line arguments. All parameters are required unless a default is specified.

```bash
cargo run --release -- \
    --coldkey "<COLDKEY mnemonic word>" \
    --hotkey "<HOTKEY mnemonic word>" \
    --netuid <NETUID> \
    [--max-cost <MAX_COST> (optional)]
```

Alternatively, after building the project, run:

```bash
./regbot --coldkey "<COLDKEY mnemonic word>" --hotkey "<HOTKEY mnemonic word>" --netuid <NETUID>"
```

### Example Usage

```bash
cargo run --release -- \
    --coldkey "garden cherry orbit fabric loyal drift wisdom ocean cactus enrich drama shell" \
    --hotkey "orange vivid canyon crisp umbrella talent eagle fossil shrimp velvet adapt breeze" \
    --netuid 1 \
    [--max-cost 6000000000]
```

## Parameters Explained

- `--coldkey`: Coldkey seed phrase or URI (used for signing and authentication).
- `--hotkey`: Hotkey seed phrase or URI (the account to be registered).
- `--netuid`: Network UID (u16, identifies the subnet/network).
- `--max-cost`: Maximum allowed recycle cost (default: 5000000000, prevents expensive transactions).
- `--chain-endpoint`: WebSocket endpoint for blockchain (default: `wss://entrypoint-finney.opentensor.ai:443`).
- `--seed`: Optional seed value for deterministic key generation.

## Output and Logging

The bot provides detailed logs for every step, including cost checks, transaction attempts, event parsing, and errors. Logs are printed to the console and can be filtered by log level (default: INFO).

Successful registration will be confirmed in the logs, along with block and event details.

## Dependencies

Key dependencies include:
- [chrono](https://crates.io/crates/chrono)
- [chrono-tz](https://crates.io/crates/chrono-tz)
- [parity-scale-codec](https://crates.io/crates/parity-scale-codec)

See `Cargo.toml` for the full list and versions.

## License

MIT License. See `LICENSE` for details.

## Author

See `Cargo.toml` for authorship information.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests for bug fixes, feature enhancements, or documentation improvements.

## Resources

- [Bittensor Documentation](https://bittensor.com/docs)
- [Rust Programming Language](https://www.rust-lang.org/)
- [Substrate Blockchain Framework](https://substrate.io/)

## Project Background & Motivation

Bittensor is a decentralized, blockchain-based protocol for machine intelligence. Registering hotkeys is a fundamental step for participating in the network, whether for mining, staking, or running validators. Manual registration can be error-prone and slow, especially for large-scale operations. `regbot` automates this process, providing reliability, repeatability, and transparency for node operators and developers.

For questions, issues, or contributions, please open an issue or pull request on the repository.
