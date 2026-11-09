use apex_core::models::ThinkingDepth;
use apex_core::speech_clean::clean_text_for_speech;
use apex_core::think_parser::{ThinkEvent, ThinkParser};
use clap::Parser;
use crossterm::execute;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use futures::StreamExt;
use std::io::{stdout, Write};

#[derive(Parser, Debug)]
#[command(author, version, about = "▲ Apex Phi-14B Rust CLI")]
struct Cli {
    /// Initial prompt to run
    #[arg(short, long)]
    prompt: Option<String>,

    /// Enable live DuckDuckGo web search & grounding
    #[arg(short, long)]
    web: bool,

    /// Speak the synthesized final answer via macOS TTS
    #[arg(short, long)]
    voice: bool,

    /// Thinking depth: fast, standard, or deep
    #[arg(short, long, default_value = "standard")]
    depth: String,

    /// Apex daemon server URL
    #[arg(short, long, default_value = "http://localhost:8000")]
    server: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    println!("▲ Apex-Phi-Think-14B (Rust Edition)");
    println!(
        "  Server: {} | Web: {} | Voice: {} | Depth: {}\n",
        args.server, args.web, args.voice, args.depth
    );

    let client = reqwest::Client::new();

    if let Some(prompt) = args.prompt {
        run_query(
            &client,
            &args.server,
            &prompt,
            args.web,
            args.voice,
            &args.depth,
        )
        .await?;
    } else {
        println!("Type your query (or 'exit' to quit):\n");
        let mut rl = std::io::stdin().lines();
        loop {
            print!("> ");
            stdout().flush()?;
            if let Some(Ok(line)) = rl.next() {
                let trimmed = line.trim();
                if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                    break;
                }
                if !trimmed.is_empty() {
                    run_query(
                        &client,
                        &args.server,
                        trimmed,
                        args.web,
                        args.voice,
                        &args.depth,
                    )
                    .await?;
                    println!("\n");
                }
            } else {
                break;
            }
        }
    }

    Ok(())
}

async fn run_query(
    client: &reqwest::Client,
    server: &str,
    query: &str,
    web: bool,
    voice: bool,
    depth: &str,
) -> anyhow::Result<()> {
    let endpoint = format!("{}/api/chat/stream", server.trim_end_matches('/'));
    let think_depth = match depth.to_lowercase().as_str() {
        "fast" => ThinkingDepth::Fast,
        "deep" => ThinkingDepth::Deep,
        _ => ThinkingDepth::Standard,
    };

    let payload = serde_json::json!({
        "messages": [
            { "role": "user", "content": query }
        ],
        "web_search": web,
        "think_depth": think_depth
    });

    let resp = client.post(&endpoint).json(&payload).send().await;

    match resp {
        Ok(res) if res.status().is_success() => {
            let mut byte_stream = res.bytes_stream();
            let mut parser = ThinkParser::new();
            let mut full_answer = String::new();
            let mut line_buf = String::new();
            let mut in_think_display = false;

            while let Some(item) = byte_stream.next().await {
                if let Ok(bytes) = item {
                    line_buf.push_str(&String::from_utf8_lossy(&bytes));
                    while let Some(pos) = line_buf.find("\n\n") {
                        let block = line_buf[..pos].to_string();
                        line_buf = line_buf[pos + 2..].to_string();

                        for line in block.lines() {
                            if let Some(stripped) = line.strip_prefix("data: ") {
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(stripped)
                                {
                                    if let Some(token) = val.get("token").and_then(|v| v.as_str()) {
                                        let events = parser.feed(token);
                                        for ev in events {
                                            match ev {
                                                ThinkEvent::ThinkChunk(chunk) => {
                                                    if !in_think_display {
                                                        execute!(
                                                            stdout(),
                                                            SetForegroundColor(Color::DarkGrey)
                                                        )?;
                                                        print!("🧠 [Thinking]: ");
                                                        in_think_display = true;
                                                    }
                                                    print!("{}", chunk);
                                                    stdout().flush()?;
                                                }
                                                ThinkEvent::ThinkEnd => {
                                                    if in_think_display {
                                                        execute!(stdout(), ResetColor)?;
                                                        println!("\n");
                                                        in_think_display = false;
                                                    }
                                                }
                                                ThinkEvent::AnswerChunk(chunk) => {
                                                    full_answer.push_str(&chunk);
                                                    print!("{}", chunk);
                                                    stdout().flush()?;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            execute!(stdout(), ResetColor)?;
            println!();

            if voice && !full_answer.is_empty() {
                let cleaned = clean_text_for_speech(&full_answer);
                if !cleaned.is_empty() {
                    let _ = std::process::Command::new("say").arg(&cleaned).output();
                }
            }
        }
        _ => {
            println!("⚠️  Could not connect to Apex Daemon at {}. Ensure 'cargo run -p apex-daemon' is running.", server);
        }
    }

    Ok(())
}
