# Rust CLI Chat

A secure command-line chat application implemented in Rust with end-to-end encryption and Tor support.

## Security Features

- **End-to-End Encryption (E2EE)**: Using X25519 for key exchange and XChaCha20-Poly1305 for message encryption
- **Perfect Forward Secrecy**: Session-level PFS when using ephemeral keys
- **Message Integrity**: Poly1305 authentication tags
- **Public Key Verification**: Simple fingerprint verification via SHA256(pubkey)
- **Tor Network**: Hidden service support for enhanced privacy

## Protocol Details

1. **Key Generation**: Each party generates X25519 StaticSecret and PublicKey
2. **Key Exchange**: Parties exchange 32-byte public keys via length-prefixed frames
3. **Shared Secret**: Each party computes DH(private, peer_pub) → 32 bytes
4. **Key Derivation**: HKDF-SHA256(shared) → 32-byte AEAD key
5. **Message Encryption**: XChaCha20Poly1305 with 24-byte random nonce
6. **Message Format**: nonce || ciphertext
7. **Frame Format**: 4-byte big-endian length prefix + payload

## Requirements

- Rust 1.70+
- Tor Expert Bundle

## Installation

1. Clone the repository:
```bash
git clone https://github.com/mavxa/rust-cli-chat.git
cd rust-cli-chat
```

2. Build the project:
```bash
cargo build --release
```

### Running the Server

1. Configure Tor (optional):
   - Install Tor Expert Bundle
   - Copy `tor/torrc.example` to your Tor configuration directory
   - Update paths in the config if needed

2. Start the server:
```bash
./scripts/run_local.bat  # Windows
```

The server will print its .onion address (if using Tor) or local address.

### Running the Client

```bash
cargo run --release --bin client [SERVER_ADDRESS]
```

Replace [SERVER_ADDRESS] with either:
- .onion address (for Tor connections)
- IP:PORT (for direct connections)