# Titan HFT (SHARK V7 RELOADED)

[![CI](https://github.com/quantbotsoftware/titan-hft/actions/workflows/ci.yml/badge.svg)](https://github.com/quantbotsoftware/titan-hft/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) [![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.23992530.svg)](https://doi.org/10.5281/zenodo.23992530) [![Hub](https://img.shields.io/badge/hub-bot--tradingsoftware.com-0a7)](https://bot-tradingsoftware.com/)

![License: MIT](https://img.shields.io/badge/license-MIT-10b981) ![Price: Free](https://img.shields.io/badge/price-free-3b82f6) [![Donations: Good Samaritans](https://img.shields.io/badge/donations-good%20samaritans-ec4899)](https://ko-fi.com/quantbotsoftware)

**Free & open-source** algorithmic trading desktop application built with Tauri 1.x (Rust backend + high-performance vanilla JS frontend), WGPU compute shaders and a 7-layer signal Oracle.

> 🆓 **Free forever.** No license fees, no paywall, no locked features.
> ❤️ If Titan saved you time, taught you something, or you simply believe good tools should be free — consider supporting the project as a good samaritan. See [Support the project](#️-support-the-project).

> [!WARNING]
> **Provided AS-IS — no liability, no support.** Trading involves substantial risk of loss. Titan HFT is educational/research software, **not financial advice**. The author accepts **no responsibility** for any losses or damages and provides **no assistance** (installation, configuration, trading or otherwise). The documentation in this repo is the only resource. Always start with an Alpaca **paper** account.

---

## 📋 System Prerequisites

To compile the application, you need the standard Rust toolchain and the platform-specific Tauri dependencies.

### 1. Install Rust
Install the Rust toolchain via rustup:
* **All Platforms:** [https://rustup.rs/](https://rustup.rs/)

### 2. Platform-Specific System Dependencies

#### 🪟 Windows
* **C++ Build Tools:** Download and install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the "Desktop development with C++" workload during installation.
* **WebView2:** Installed by default on Windows 10/11. If missing, download it from Microsoft.

#### 🍎 macOS
* **Xcode Command Line Tools:** Run the following command in terminal:
  ```bash
  xcode-select --install
  ```

#### 🐧 Linux (Debian/Ubuntu)
Install the required development libraries:
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.0-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### 3. GPU Drivers (WGPU Support)
The application utilizes WGPU compute shaders for parallel agent simulations on the GPU. Ensure your system has active drivers supporting:
* **Windows:** DirectX 12 or Vulkan
* **macOS:** Metal (supported out-of-the-box on Apple Silicon and modern Intel Macs)
* **Linux:** Vulkan drivers (e.g., `mesa-vulkan-drivers` or proprietary NVIDIA drivers)

---

## 🛠️ Build & Compilation Steps

Since the frontend is built using highly optimized, zero-dependency vanilla JavaScript, **Node.js/npm is not required to build the project**. 

### Step 1: Sync Frontend Assets
Tauri is configured to embed the static frontend assets from the `dist/` directory at compile-time. Before building, you must sync/copy the `src/` directory to a new `dist/` directory at the root of the project:

* **Linux / macOS:**
  ```bash
  cp -r src dist
  ```
* **Windows (PowerShell):**
  ```powershell
  Copy-Item -Recurse -Force src dist
  ```

### Step 2: Compile the Binary
Navigate into the `src-tauri` directory and compile the release binary:

```bash
cd src-tauri
cargo build --release --features custom-protocol
```

The compiled executable will be generated at:
`src-tauri/target/release/titan_hft_rust` (or `titan_hft_rust.exe` on Windows).

### Alternative: Compile using Tauri CLI
If you prefer to package the application as a native installer (e.g., `.msi` on Windows, `.dmg` on macOS, `.deb` on Linux), install the Tauri CLI and run the bundler:

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^1"

# Bundle application
cargo tauri build
```

---

## ⚙️ Post-Build Configuration

> [!IMPORTANT]
> **No API keys are included.** Titan HFT does not ship, share or provide Alpaca credentials. Every user must create their own (free) account at [alpaca.markets](https://alpaca.markets), generate their own **paper-trading** keys and enter them locally. Your keys never leave your machine.

The simplest path is to launch the application and enter your Alpaca credentials in the in-app **Settings** panel; they are persisted in `data/titan_keys.json` inside the application's data directory.

Alternatively, you can pre-seed the credentials with a `config.toml` file:

1. Copy `config.toml.example` to `config.toml` inside the application's data directory. When running a source build this is `src-tauri/` (the app resolves it as the parent of the `target/` folder). You can override the data directory with the `TITAN_DATA_DIR` environment variable.
2. Fill in your Alpaca API credentials:
   ```toml
   [alpaca]
   key_id = "YOUR_ALPACA_KEY_ID"
   secret = "YOUR_ALPACA_SECRET"
   base_url = "https://paper-api.alpaca.markets"  # use https://api.alpaca.markets for live trading
   ```
3. Run the compiled binary. `config.toml` is only consulted when `data/titan_keys.json` does not exist yet. All strategy and risk parameters are configured inside the application, not in this file.

---

## ✅ Running the Test Suite

```bash
cd src-tauri
cargo test --workspace
```

The workspace ships with 500+ unit and integration tests covering the indicator engine, risk gates, SmartTrend state machine, order execution flow and persistence layer.

---

## 📚 Documentation

All manuals are provided in English, with Romanian translations suffixed `_RO`.

| Document | Scope |
|----------|-------|
| `TITAN_MANUAL.md` | Complete user & operator manual (UI, settings, loops, persistence, web dashboard) |
| `TITAN_HFT_ENGINE_MANUAL.md` | Trading engine internals: autopilot, SmartTrend, risk gates, LLM sentiment, Telegram |
| `TITAN_ORACLE.md` | 7-layer Oracle signal engine (Kalman, Entropy, Hurst, Lyapunov, Wavelet, VPIN, BOCPD) |
| `GPU_NEXUS_MANUAL.md` | WGPU compute pipeline and Nexus consensus kernel |
| `MARKET_DISCOVERY_MANUAL.md` | Scanner, watchlist and market discovery workflow |
| `TITAN_SIGNAL_SCALPING_MASTER_GUIDE.md` | Discretionary scalping methodology using Titan signals |
| `TITAN_SIGNAL_SCALPING_ADVANCED_PLAYBOOK.md` | Advanced playbook and checklists |
| `TITAN_TRADE_JOURNAL_TEMPLATE.md` / `.csv` | Trade journal templates |

## 📁 Repository Layout

```
src/                     Frontend (vanilla JS, Tauri IPC, Lightweight Charts)
src-tauri/src/           Rust backend: commands/, loops/, engine, GPU shaders (*.wgsl)
src-tauri/crates/        Workspace crates: titan-types, titan-risk, titan-genetic, titan-indicators
src-tauri/tests/         Integration tests
src-tauri/benches/       Criterion benchmarks
examples/backtests/      Sample output of the built-in backtesting engine (QBTS, 1d)
build.sh                 Full release build + bundle helper (Linux)
```

---

## ❤️ Support the project

Titan HFT is given away for free. Development, testing and GPU hours are funded only by voluntary donations from people who find it useful.

| Channel | Link |
|---------|------|
| Ko-fi ($1+ Voluntary Tip) | [https://ko-fi.com/quantbotsoftware](https://ko-fi.com/quantbotsoftware) |
| Website | https://bot-tradingsoftware.com/#donate |

Donations are voluntary gifts: not a purchase, non-refundable, and they do **not** entitle anyone to support, assistance, warranty or updates. ⭐ Starring the repo and sharing it also helps a lot.

## ⚖️ License

[MIT](LICENSE.txt) — free to use, modify and redistribute. Third-party components: TradingView Lightweight Charts™ (Apache-2.0), Share Tech Mono font (SIL OFL 1.1). See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and `LICENSES/`.

Charts powered by [TradingView Lightweight Charts™](https://www.tradingview.com/lightweight-charts/).

## ™ Trademarks

Alpaca is a trademark of AlpacaDB, Inc. TradingView and Lightweight Charts are trademarks of TradingView, Inc. Titan HFT is an independent project and is **not affiliated with, endorsed by or sponsored by** these companies. Brand names are used only to describe compatibility.

Full legal terms, risk disclaimer and privacy policy: https://bot-tradingsoftware.com/legal.html


## 🖼️ UI Reference Screenshots (Compiled Desktop Confirmation)

> *Illustrative engineering confirmation only — showing the compiled Rust + Tauri + WGPU desktop interface; not a financial or performance claim.*

- 🌐 **Full Interactive UI Gallery:** [https://bot-tradingsoftware.com/#screenshots](https://bot-tradingsoftware.com/#screenshots)

![Titan HFT Dashboard](https://bot-tradingsoftware.com/img/screenshots/titan-hft-1-dashboard.png)

![Titan HFT 60 FPS Pro Chart & Orderbook Heatmap](https://bot-tradingsoftware.com/img/screenshots/titan-hft-2-chart-60fps.png)

![Titan HFT GPU Nexus 150K Agents & Safety](https://bot-tradingsoftware.com/img/screenshots/titan-hft-3-gpu-nexus.png)


## 📄 Additional Documentation on Request

Seven full English + Romanian manuals are already bundled in this repository. If you are studying a specific subsystem and need additional existing archival documentation, you may open a `[Docs Request]` ticket (if available, shared strictly AS-IS, with zero technical support).


## ✅ Continuous Integration

Every push to `main` runs **`cargo test --workspace --locked` (549 unit + integration tests across the engine, risk guards, journal, oracle and workspace crates)** on Ubuntu 22.04 with the Tauri v1 toolchain (Rust pinned to 1.93.0 — see `rust-toolchain.toml`), plus `node --check` on all 34 frontend scripts and a static smoke test of `index.html`. Tests are hermetic: `cfg(test)` redirects all data/journal paths to a temporary directory.

Workflow: [`.github/workflows/ci.yml`](.github/workflows/ci.yml) · runs: [https://github.com/quantbotsoftware/titan-hft/actions](https://github.com/quantbotsoftware/titan-hft/actions). No secrets, no broker or network calls, no telemetry — fork it and the same checks run for you.


## 📊 Paper-Trading Track Record (real numbers, losses included)

Titan has **no meaningful autonomous paper-trading record yet** (it has mostly been used as an execution / visualisation / GPU-simulation engine). We say so plainly instead of showing a cherry-picked backtest.

- 🇬🇧 [TITAN_PAPER_TRACK_RECORD.md](TITAN_PAPER_TRACK_RECORD.md) · 🇷🇴 [TITAN_PAPER_TRACK_RECORD_RO.md](TITAN_PAPER_TRACK_RECORD_RO.md)
- Online: [https://bot-tradingsoftware.com/#track-record](https://bot-tradingsoftware.com/#track-record)

> Paper trading ≠ live trading. Simulated fills are optimistic. Nothing here is financial advice or a performance claim.
