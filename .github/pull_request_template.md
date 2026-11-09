## Description
Briefly describe the change and motivation.

## Subsystem Impact
- [ ] `apex-core`: Token parsing, speech normalization, model primitives
- [ ] `apex-web`: DuckDuckGo scraping, HTML extraction, grounding engine
- [ ] `apex-daemon`: Axum HTTP/SSE server, official A2A v1.0 JSON-RPC
- [ ] `apex-cli`: Terminal interactive UI, streaming output
- [ ] `apex-harness`: Pass-state gating, features.json primitives

## Harness Verification Gate
- [ ] Unit & integration tests pass (`cargo test --workspace`)
- [ ] Harness certified (`cargo run -p apex-harness --bin harness-verify -- --all`)
- [ ] Clippy warnings free (`cargo clippy --workspace --all-targets -- -D warnings`)
- [ ] Rustfmt formatted (`cargo fmt --all -- --check`)
