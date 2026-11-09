# Apex Engine (Rust) — Agent Operating Guide

## 1. Project Purpose & Scope
Unified, high-performance Rust engine powering:
- `apex-core`: Zero-allocation `<think>` reasoning streaming state machine & audio cleanup.
- `apex-harness`: WalkingLabs 5-subsystem harness with feature primitives & 3-layer termination gating.
- `apex-web`: DuckDuckGo autonomous web retrieval & DOM documentation scraper.
- `apex-daemon`: Axum 0.8 HTTP/SSE gateway with official Linux Foundation `a2a-server-lf` integration.
- `apex-cli`: Terminal interactive REPL with colorized `<think>` drawer & macOS voice audio.

## 2. Tech Stack & Environment
- **Language:** Rust 1.85+ (MSRV) / Edition 2021
- **Async Runtime:** Tokio 1.40+ (multi-threaded work-stealing)
- **Web & A2A:** Axum 0.8, Tower-HTTP, `a2a-lf` (v0.4), `a2a-server-lf` (v0.5)
- **Local AI:** Ollama (`http://localhost:11434`, model: `apex-phi-think-14b`)
- **System Audio:** macOS `/usr/bin/say`

## 3. Non-Negotiable Hard Constraints (Invariants)
1. **Pass-State Gating:** Features in `features.json` can ONLY be marked `"passing"` by the harness verifier executing the registered verification command. Never edit `"state": "passing"` manually.
2. **Completion Priority:** Correctness $\rightarrow$ Test Verification $\rightarrow$ Performance $\rightarrow$ Styling. No refactoring before core functionality passes.
3. **Memory Ceiling:** Background daemon resident RAM must remain $< 20\text{ MB}$.
4. **Wire Compatibility:** All A2A endpoints must strictly conform to the official Linux Foundation specification at `https://a2a-protocol.org/latest/`.

## 4. Verification Commands (Highest-ROI Feedback Loop)
- **Layer 1 (Static Check & Lint):**
  ```bash
  cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings
  ```
- **Layer 2 (Unit & Library Tests):**
  ```bash
  cargo test --workspace --lib
  ```
- **Layer 3 (Harness Feature Verifier):**
  ```bash
  cargo run -p apex-harness --bin harness-verify -- --all
  ```
