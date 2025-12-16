# Zapatos

A minimalist Aptos node implementation focused on state synchronization and streaming.

## Project Structure

```
zapatos/
├── zap/          # Main node crate
│   ├── src/
│   │   ├── config/       # Node configuration
│   │   ├── crypto/       # Cryptographic primitives
│   │   ├── network/      # P2P networking with Noise protocol
│   │   ├── state_sync/   # State synchronization
│   │   ├── storage/      # Storage layer
│   │   ├── types/        # Core types (Transaction, StateKey, etc.)
│   │   └── mode.rs       # Node operational modes
│   └── fixtures/         # Test fixtures (genesis.blob, waypoint.txt)
└── types/        # Aptos types library
```

## Node Modes

The `node` binary supports three operational modes:

### 1. **Streaming Mode** (Default)
In-memory state synchronization with debug logging. No persistent storage.

**Use case:** Monitoring blockchain state changes in real-time without disk overhead.

**Features:**
- No database required
- Real-time state change logging
- Minimal resource footprint
- Perfect for debugging and monitoring

### 2. **FullNode Mode** (Coming Soon)
Full node with persistent storage and complete state sync.

### 3. **Validator Mode** (Coming Soon)
Validator node with consensus participation.

## Quick Start

### Build

```bash
cargo build --release -p zap
```

### Run in Streaming Mode

```bash
# Start with mainnet genesis and waypoint
./target/release/node \
  --mode stream \
  --genesis-file zap/fixtures/mainnet/genesis.blob \
  --waypoint-file zap/fixtures/mainnet/waypoint.txt

# Connect to a peer
./target/release/node \
  --mode stream \
  --peer-address "127.0.0.1:6180" \
  --peer-id "your_peer_public_key_hex"
```

### Expected Output

```
Starting Aptos Node in stream mode...
[INFO] Starting node in Streaming mode
[INFO] Loaded waypoint: 1638030465:c2b8b05061483c8d89426020fc3e6282746bd68299908113c265823860a5476e
[INFO] Loaded genesis (996225 bytes)
[STREAM] Node initialized in streaming mode
[STREAM] Waiting for state sync updates...
[STREAM] Node running in streaming mode (Press Ctrl+C to exit)
```

## CLI Options

```
Options:
  -m, --mode <MODE>              Node operational mode [default: stream]
                                 Values: stream, fullnode, validator
  -f, --config <CONFIG>          Path to node configuration file
      --peer-address <ADDRESS>   Peer address (e.g., "127.0.0.1:6180")
      --peer-id <ID>             Peer public key (hex)
      --genesis-file <PATH>      Path to genesis blob
      --waypoint-file <PATH>     Path to waypoint file
  -h, --help                     Print help
  -V, --version                  Print version
```

## Development

### Run Tests

```bash
cargo test -p zap
```

### Genesis Loading Tests

The project includes tests for loading and deserializing mainnet genesis and waypoint files:

```bash
cargo test -p zap test_load_mainnet_fixtures
```

## Architecture

### Streaming Mode Architecture

```
┌─────────────────────────────────────────┐
│           Node (Streaming)              │
├─────────────────────────────────────────┤
│  ┌─────────────┐     ┌───────────────┐ │
│  │   Network   │────▶│  State Sync   │ │
│  │   (Noise)   │     │               │ │
│  └─────────────┘     └───────┬───────┘ │
│                              │         │
│                              ▼         │
│                      ┌───────────────┐ │
│                      │ Memory Store  │ │
│                      │  (In-Memory)  │ │
│                      └───────┬───────┘ │
│                              │         │
│                              ▼         │
│                      ┌───────────────┐ │
│                      │ Debug Logger  │ │
│                      │  [STREAM]     │ │
│                      └───────────────┘ │
└─────────────────────────────────────────┘
```

## License

Apache 2.0
