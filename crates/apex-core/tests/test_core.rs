use apex_core::{clean_text_for_speech, ThinkEvent, ThinkParser};

#[test]
fn test_think_parser_normal_transition() {
    let mut parser = ThinkParser::new();
    let events = parser.feed("Reasoning about parity...\n</think>\n\nEven number.");
    
    assert!(events.contains(&ThinkEvent::ThinkChunk("Reasoning about parity...\n".to_string())));
    assert!(events.contains(&ThinkEvent::ThinkEnd));
    assert!(events.contains(&ThinkEvent::AnswerChunk("Even number.".to_string())));
    assert!(!parser.is_in_think());
}

#[test]
fn test_think_parser_split_boundary_chunk() {
    let mut parser = ThinkParser::new();
    
    // Chunk 1 ends with partial closing tag "</th"
    let events1 = parser.feed("Deriving invariant: x > 0 </th");
    assert!(events1.contains(&ThinkEvent::ThinkChunk("Deriving invariant: x > 0 ".to_string())));
    assert!(parser.is_in_think());

    // Chunk 2 completes the tag with "ink>\n\nVerified."
    let events2 = parser.feed("ink>\n\nVerified.");
    assert!(events2.contains(&ThinkEvent::ThinkEnd));
    assert!(events2.contains(&ThinkEvent::AnswerChunk("Verified.".to_string())));
    assert!(!parser.is_in_think());
}

#[test]
fn test_speech_clean_strips_think() {
    let raw = "<think>\nStep 1: Check edge case n = 0\nStep 2: Base induction\n</think>\n\nThe function returns 1.";
    let cleaned = clean_text_for_speech(raw);
    assert_eq!(cleaned, "The function returns 1.");
}

#[test]
fn test_speech_clean_strips_code_fences_and_links() {
    let raw = "Here is the implementation:\n```rust\nfn main() { println!(\"Hi\"); }\n```\nSee [Documentation](https://docs.rs).";
    let cleaned = clean_text_for_speech(raw);
    assert!(!cleaned.contains("fn main()"));
    assert!(cleaned.contains("Code block omitted for voice summary."));
    assert!(cleaned.contains("Documentation"));
    assert!(!cleaned.contains("https://docs.rs"));
}
