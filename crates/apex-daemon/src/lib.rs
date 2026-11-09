pub mod card;
pub mod executor;
pub mod ollama;
pub mod routes;

pub use card::build_apex_agent_card;
pub use executor::ApexAgentExecutor;
pub use ollama::OllamaClient;
pub use routes::{create_router, AppState};

use std::net::SocketAddr;
use std::sync::Arc;

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let base_url = format!("http://localhost:{}", port);
    let state = AppState {
        ollama: Arc::new(OllamaClient::default()),
        grounding: Arc::new(apex_web::GroundingEngine::default()),
        base_url,
    };

    let app = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("🚀 Apex Daemon listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
