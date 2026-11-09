use a2a::{AgentCapabilities, AgentCard, AgentInterface, AgentSkill};

pub fn build_apex_agent_card(base_url: &str) -> AgentCard {
    let rpc_url = format!("{}/rpc", base_url.trim_end_matches('/'));
    
    AgentCard {
        name: "ApexPhi-Think-14B".to_string(),
        description: "Frontier Mathematical Reasoning, Autonomous Web Research & Voice Synthesis Engine on Microsoft Phi-4 (14.7B)".to_string(),
        version: "1.0.0".to_string(),
        supported_interfaces: vec![
            AgentInterface::new(rpc_url, "JSON-RPC"),
        ],
        capabilities: AgentCapabilities {
            streaming: Some(true),
            push_notifications: Some(false),
            extensions: None,
            extended_agent_card: Some(false),
        },
        default_input_modes: vec!["text/plain".to_string(), "application/json".to_string()],
        default_output_modes: vec!["text/plain".to_string(), "text/event-stream".to_string(), "application/json".to_string()],
        skills: vec![
            AgentSkill {
                id: "mathematical_proof".to_string(),
                name: "Mathematical Reasoning & Invariant Proving".to_string(),
                description: "Performs deep step-by-step invariant derivations inside <think> blocks before outputting conclusions.".to_string(),
                tags: vec!["reasoning".to_string(), "math".to_string(), "invariants".to_string(), "phi4".to_string()],
                examples: None,
                input_modes: None,
                output_modes: None,
                security_requirements: None,
            },
            AgentSkill {
                id: "web_grounded_research".to_string(),
                name: "Autonomous Web Grounding & Documentation Scraping".to_string(),
                description: "Autonomously queries DuckDuckGo, scrapes official documentation, and synthesizes grounded citations.".to_string(),
                tags: vec!["web".to_string(), "retrieval".to_string(), "citations".to_string()],
                examples: None,
                input_modes: None,
                output_modes: None,
                security_requirements: None,
            },
            AgentSkill {
                id: "voice_audio_synthesis".to_string(),
                name: "Voice Audio Enunciation & Synthesis".to_string(),
                description: "Synthesizes spoken natural audio summaries free of raw Markdown or unpronounceable code syntax.".to_string(),
                tags: vec!["voice".to_string(), "tts".to_string(), "audio".to_string()],
                examples: None,
                input_modes: None,
                output_modes: None,
                security_requirements: None,
            },
        ],
        provider: None,
        documentation_url: Some("https://github.com/a2aproject/A2A".to_string()),
        icon_url: None,
        security_schemes: None,
        security_requirements: None,
        signatures: None,
    }
}
