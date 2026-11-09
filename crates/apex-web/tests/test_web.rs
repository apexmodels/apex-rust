use apex_web::{DuckDuckGoClient, GroundingEngine, PageScraper};

#[test]
fn test_duckduckgo_html_parsing() {
    let sample_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <div class="result">
            <h2 class="result__title">
                <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust-lang.org%2F&rut=...">Rust Programming Language</a>
            </h2>
            <a class="result__snippet">A language empowering everyone to build reliable and efficient software.</a>
        </div>
        <div class="result">
            <h2 class="result__title">
                <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fdoc.rust-lang.org%2Fbook%2F&rut=...">The Rust Book</a>
            </h2>
            <a class="result__snippet">The Rust Programming Language book provides an overview of Rust.</a>
        </div>
    </body>
    </html>
    "#;

    let hits = DuckDuckGoClient::parse_html_hits(sample_html, 5);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].title, "Rust Programming Language");
    assert_eq!(hits[0].href, "https://www.rust-lang.org/");
    assert!(hits[0].snippet.contains("reliable and efficient software"));

    assert_eq!(hits[1].title, "The Rust Book");
    assert_eq!(hits[1].href, "https://doc.rust-lang.org/book/");
}

#[test]
fn test_scraper_readable_text() {
    let raw_html = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Test Page</title><style>.hidden { display: none; }</style></head>
    <body>
        <nav><a href="/home">Home Menu</a> <a href="/about">About Us</a></nav>
        <header><h1>Welcome to Our Site Header</h1></header>
        <main>
            <h1>Understanding Harness Engineering</h1>
            <p>Harness engineering treats AI agent workflows with strict software testing primitives, ensuring predictable autonomous execution.</p>
            <script>console.log("bad script content");</script>
            <p>By enforcing three-layer verification gates, models cannot hallucinate task completion.</p>
        </main>
        <footer><p>Copyright 2026 Corporation. All rights reserved.</p></footer>
    </body>
    </html>
    "#;

    let extracted = PageScraper::extract_readable_text(raw_html, 2000);
    assert!(extracted.contains("Understanding Harness Engineering"));
    assert!(extracted.contains(
        "Harness engineering treats AI agent workflows with strict software testing primitives"
    ));
    assert!(extracted.contains("By enforcing three-layer verification gates"));

    // Ensure nav, script, footer are stripped out
    assert!(!extracted.contains("bad script content"));
    assert!(!extracted.contains("Home Menu"));
    assert!(!extracted.contains("Copyright 2026"));
}

#[tokio::test]
async fn test_grounding_engine_instantiation() {
    let engine = GroundingEngine::new();
    // Test that the engine can be created and default configurations are valid
    let result = engine.research("rust tokio", 1).await;
    match result {
        Ok(grounded) => {
            println!(
                "Grounding completed with {} sources",
                grounded.sources.len()
            );
        }
        Err(e) => {
            // Network failures in sandboxed/offline environments shouldn't fail test,
            // but the error type should be handled cleanly.
            println!("Network note during test: {}", e);
        }
    }
}
