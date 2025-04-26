# VoiceBBS

**VoiceBBS** is a modern re-imagining of a classic voice-based Bulletin Board System (BBS), built using Rust.  
It allows users to call into the system via traditional phone lines (using SIP) and interact with audio-based posts and menus.

## Features

- **Warp HTTP Server** — Serves a basic web dashboard (port 8080).
- **SIP Server over UDP** — Listens on UDP port 5060 for incoming SIP INVITE requests.
- **Simple 200 OK Response** — Currently accepts incoming calls and responds properly per SIP standards.
- **Future Plans** — Expand into interactive voice menus, posting responses, protected areas, and more.

## Requirements

- Rust (1.81 or higher recommended)
- Ubuntu 22.04 LTS (or any Linux system with Rust toolchain)
- SIP-compatible service (e.g., SignalWire) pointing to server's public IP on UDP 5060
- (Optional) UFW firewall configured to allow ports 22 (SSH), 8080 (HTTP), and 5060 (SIP UDP)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/YOUR-GITHUB-USERNAME/voicebbs.git

# Change into the project directory
cd voicebbs

# Build the project
cargo build --release

# Run the server
./target/release/voicebbs