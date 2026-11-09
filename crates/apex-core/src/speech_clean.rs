use regex::Regex;
use std::sync::LazyLock;

static RE_CODE_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)```.*?```").expect("valid regex")
});

static RE_INLINE_CODE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"`([^`]+)`").expect("valid regex")
});

static RE_MARKDOWN_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([^\]]+)\]\([^\)]+\)").expect("valid regex")
});

static RE_HEADERS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^#+\s*").expect("valid regex")
});

static RE_SYMBOLS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\*_~]").expect("valid regex")
});

static RE_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s+").expect("valid regex")
});

/// Cleans raw LLM response text for natural-sounding Text-to-Speech (TTS).
pub fn clean_text_for_speech(raw_text: &str) -> String {
    // 1. If </think> is present, only speak text after </think>
    let text = if let Some(pos) = raw_text.find("</think>") {
        &raw_text[pos + "</think>".len()..]
    } else {
        raw_text
    };

    // 2. Strip code blocks
    let text = RE_CODE_BLOCK.replace_all(text, " Code block omitted for voice summary. ");

    // 3. Strip inline code `foo` -> foo
    let text = RE_INLINE_CODE.replace_all(&text, "$1");

    // 4. Strip markdown links [foo](url) -> foo
    let text = RE_MARKDOWN_LINK.replace_all(&text, "$1");

    // 5. Strip headers and formatting symbols
    let text = RE_HEADERS.replace_all(&text, "");
    let text = RE_SYMBOLS.replace_all(&text, "");

    // 6. Normalize whitespace
    let normalized = RE_WHITESPACE.replace_all(&text, " ");
    normalized.trim().to_string()
}
