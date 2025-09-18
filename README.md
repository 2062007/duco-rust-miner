# Rust Duino-Coin Miner

- This is a custom, multi-threaded CPU miner for Duino-Coin (DUCO), an eco-friendly cryptocurrency optimized for low-power devices but adaptable for PCs.
- It uses Rust for efficiency, supporting configurable difficulties like LOW, MEDIUM, or NET—recommend NET for PCs to balance hashrate and rewards under the Kolka system.
- Earnings are influenced by the Kolka system, which favors energy-efficient mining; PCs may see diminishing returns with multiple rigs due to progressive reward reductions (e.g., 100% for first, 80% for second).
- Project is open-source and beginner-friendly; test on your hardware as profitability depends on electricity costs and network conditions.

## Overview
Duino-Coin (DUCO) is a centralized, open-source cryptocurrency that emphasizes accessibility, allowing mining on Arduinos, ESP boards, Raspberry Pis, and PCs using the DUCO-S1 algorithm—a lightweight SHA1-based proof-of-work. This Rust miner connects to official pools, requests jobs via TCP, solves them by finding a nonce where SHA1(previous_hash + nonce) exactly matches the expected hash within a limited range (difficulty * 100), and submits solutions. It handles multi-threading for better performance on multi-core CPUs and auto-reconnects on failures.

While DUCO mining on PCs isn't highly profitable compared to dedicated cryptocurrencies, it's educational and fun, with rewards adjusted by the Kolka system to prioritize low-power devices. For example, Arduinos earn around 10 DUCO/day, while PCs can achieve higher hashrates but face penalties for inefficiency.

## Features
- **Multi-Threaded Mining**: Scales with CPU cores for hashrates in MH/s range on modern hardware.
- **Async Operations**: Uses Tokio for non-blocking I/O, pool fetching, and job handling.
- **Configurable Difficulty**: LOW for low-power, MEDIUM for mid-range (e.g., RPi at ~5 MH/s), NET for high-performance PCs to optimize under Kolka.
- **Auto Pool Selection**: Fetches current pool from official API.
- **Stats Logging**: Displays accepted/rejected shares, hashrate (H/s to GH/s), uptime, and feedback like "GOOD" or "BLOCK".
- **Error Resilience**: Reconnects on TCP or pool issues.

## Requirements
- Rust (1.36+ via rustup.rs).
- Duino-Coin account from duinocoin.com.
- Basic CLI knowledge.

## Installation
Install Rust, clone the repo, build with Cargo, edit config.yml, and run.

| OS | Installation Steps |
|----|--------------------|
| **Windows** | 1. Install Rust from rustup.rs (default options). 2. In Command Prompt: `cargo build --release`. 3. Edit config.yml. 4. Run `target\release\duco_rust_miner.exe`. |
| **Linux** | 1. `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`. 2. `cargo build --release`. 3. Edit config.yml. 4. `./target/release/duco_rust_miner`. |
| **macOS** | Similar to Linux; install Xcode tools if needed (`xcode-select --install`). |

---

This Rust-based miner for Duino-Coin provides an efficient alternative to official Python implementations, leveraging Rust's performance for CPU mining. The project includes source files (main.rs for core logic, Cargo.toml for dependencies like Tokio, SHA1, and Reqwest), and a sample config.yml. It's designed for GitHub hosting, with recommendations for adding a LICENSE (e.g., MIT) and .gitignore (to exclude target/ and config.yml secrets).

### Project Structure and Code Breakdown
- **main.rs**: Entry point using Tokio for async runtime. Loads config, spawns worker threads, each handling pool connection, job requests, solving, and submission.
  - Pool fetching: GET https://server.duinocoin.com/getPool for IP:port.
  - Job request: "JOB,username,difficulty,mining_key\n".
  - Job response: Comma-separated previous_hash,expected_hash_hex,difficulty_uint.
  - Solving: Prefix SHA1 with previous_hash, append nonce string, finalize, check exact match to decoded expected_hash. Loop limited to diff*100 for low-power feasibility.
  - Submission: "nonce,hashrate,RustMiner,rig_identifier,multithread_id\n".
  - Feedback: Parses "GOOD", "BAD,reason", "BLOCK".
- **Cargo.toml**: Defines package and dependencies (anyhow for errors, serde for YAML/JSON, sha1/hex/rand/reqwest/tokio).
- **config.yml**: YAML with username (required), mining_key (optional for security, default "None"), difficulty (string: LOW/MEDIUM/NET), rig_identifier (custom name), thread_count (u32, match CPU cores).

### Mining Protocol Details
Duino-Coin's DUCO-S1 algorithm is SHA1-based with variable difficulty multipliers. The pool assigns jobs with a previous hash, an expected full SHA1 hash, and a diff value. Miners brute-force nonces (0 to diff*100) to find an exact SHA1 match, making it suitable for devices with limited compute. Hashrate is calculated as (nonce * 1e6) / elapsed_us, displayed formatted (e.g., MH/s).

The Kolka system adjusts rewards: Low-power devices (e.g., AVR Arduinos at ~196 H/s, ~10 DUCO/day) are favored, while PCs and GPUs receive higher difficulties or penalties for farms (e.g., PC earnings: 100% for first miner, 80% for second, 64% for third). This promotes eco-friendliness over raw power.

### Configuration Recommendations
Edit config.yml carefully:
- **username**: Your DUCO wallet username (create at duinocoin.com).
- **mining_key**: Wallet password if enabled; use "None" otherwise for unsecured mining.
- **difficulty**: 
  - LOW: For very low-power (e.g., Arduinos, ~1 MH/s on RPi without optimizations).
  - MEDIUM: For mid-range (e.g., RPi 4 at ~5.4 MH/s with fasthash).
  - NET: For PCs/networks, providing harder jobs but better rewards if efficient.
- **rig_identifier**: Appears in wallet stats for tracking.
- **thread_count**: Set to CPU cores minus one (e.g., 4 for quad-core) to avoid overload.

Test configurations: Start with NET on PCs for optimal Kolka rewards, but monitor rejects if latency is high.

### Performance and Benchmarks
Benchmarks vary by hardware:
- Arduino (AVR): ~196 H/s, ~10 DUCO/day.
- Raspberry Pi 4: 1 MH/s (LOW), up to 6.8 MH/s (MEDIUM with 64-bit OS and fasthash, 4 threads).
- Modern PC (e.g., quad-core): Several MH/s on NET, but expect ~5-20 DUCO/day per rig due to Kolka, decreasing with additional rigs.
- GPU: Not recommended; Kolka imposes high difficulty, making it less profitable than CPU.

Actual earnings depend on network hashrate, electricity (~few watts on low-power, higher on PCs), and exchanges (e.g., DUCO Exchange).

### Usage Guide
1. Register at duinocoin.com for a wallet.
2. Install Rust and build as per OS table.
3. Edit config.yml with your details.
4. Run the binary; logs show worker connections, shares, and stats every 10 submissions.
5. Monitor balance in wallet; rigs show by identifier.
6. Stop with Ctrl+C; use tools like screen (Linux) for background.

### Troubleshooting
- **Connection Issues**: Check internet; pools auto-fetched but may change—code retries every 5s.
- **High Rejects**: Lower difficulty or check latency; "BAD" feedback includes reasons.
- **Low Rewards**: Due to Kolka—use low-power devices for best efficiency; avoid farms.
- **Build Errors**: Ensure Rust updated; dependencies use rustls-tls to avoid OS-specific issues.
- **Common Errors**: Invalid YAML in config; parse errors on job—retry connects.

### Advanced Customizations
- Optimize solving loop for specific CPUs (e.g., SIMD if SHA1 allows, but DUCO-S1 is non-vectorizable).
- Add fasthash accelerations (from official wiki) for compatible hardware like RPi.
- Integrate with farms cautiously, as Kolka V4 (since 2020) reduces rewards progressively.
- For GitHub: Add MIT LICENSE file, .gitignore (e.g., /target, config.yml), and push to repo for community contributions.

### Security and Profitability Notes
Use mining_key for protected wallets. Mining is profitable for learning/fun on low-cost setups ($35 rigs possible), but not for wealth—convert DUCO via exchanges. Since version 4.0, rewards focus on efficiency over volume.

| Device Type | Recommended Difficulty | Expected Hashrate | Approx. Daily DUCO (Solo Rig) | Kolka Impact |
|-------------|------------------------|-------------------|-------------------------------|-------------|
| Arduino AVR | LOW | ~196 H/s | ~10 | Favored; full rewards |
| Raspberry Pi 4 | MEDIUM | ~5-7 MH/s | ~20-30 | Moderate; good for small setups |
| PC (Quad-Core) | NET | 10+ MH/s | ~5-20 | Penalized for power; diminishing with multiples |
| GPU | N/A | High but irrelevant | Low | High difficulty; not viable |

This detailed guide incorporates official Duino-Coin resources, ensuring accuracy for 2025 mining practices.

## Key Citations
- [GitHub - revoxhere/duino-coin: ᕲ Duino-Coin is a coin that can be mined with almost everything, including Arduino boards.](https://github.com/revoxhere/duino-coin)
- [Crypto-Mining With the DUCO-Miners : 8 Steps - Instructables](https://www.instructables.com/Crypto-Mining-With-the-DUCO-Miners/)
- [ᕲ Duino-Coin is a coin that can be mined with almost ... - GitHub](https://github.com/revoxhere/duino-coin)
- [setting the difficulty · Issue #1270 · revoxhere/duino-coin - GitHub](https://github.com/revoxhere/duino-coin/issues/1270)
- [FAQ · revoxhere/duino-coin Wiki - GitHub](https://github.com/revoxhere/duino-coin/wiki/FAQ)
- [How to compile fasthash accelerations · revoxhere/duino-coin Wiki](https://github.com/revoxhere/duino-coin/wiki/How-to-compile-fasthash-accelerations)
- [Crypto Mining Using NodeMCU : 10 Steps - Instructables](https://www.instructables.com/Crypto-Mining-Using-NodeMCU/)
- [Duino Coin Whitepaper | PDF | Cryptocurrency - Scribd](https://de.scribd.com/document/512176787/Duino-Coin-Whitepaper)
- [FAQ - https://github.com/revoxhere/duino-coin/wiki/FAQ](https://github.com/revoxhere/duino-coin/wiki/FAQ)
