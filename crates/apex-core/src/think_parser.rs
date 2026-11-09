#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThinkEvent {
    ThinkChunk(String),
    ThinkEnd,
    AnswerChunk(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    InThink,
    InAnswer,
}

pub struct ThinkParser {
    state: ParserState,
    buffer: String,
}

impl Default for ThinkParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ThinkParser {
    const CLOSE_TAG: &'static str = "</think>";

    pub fn new() -> Self {
        Self {
            state: ParserState::InThink,
            buffer: String::new(),
        }
    }

    /// Feeds an incoming token chunk and emits 0 or more ThinkEvents.
    pub fn feed(&mut self, chunk: &str) -> Vec<ThinkEvent> {
        let mut events = Vec::new();

        match self.state {
            ParserState::InAnswer => {
                if !chunk.is_empty() {
                    events.push(ThinkEvent::AnswerChunk(chunk.to_string()));
                }
            }
            ParserState::InThink => {
                self.buffer.push_str(chunk);

                if let Some(pos) = self.buffer.find(Self::CLOSE_TAG) {
                    let think_part = &self.buffer[..pos];
                    if !think_part.is_empty() {
                        events.push(ThinkEvent::ThinkChunk(think_part.to_string()));
                    }
                    events.push(ThinkEvent::ThinkEnd);

                    let answer_part = &self.buffer[pos + Self::CLOSE_TAG.len()..];
                    let trimmed_answer = answer_part.trim_start_matches('\n');
                    if !trimmed_answer.is_empty() {
                        events.push(ThinkEvent::AnswerChunk(trimmed_answer.to_string()));
                    }

                    self.state = ParserState::InAnswer;
                    self.buffer.clear();
                } else {
                    // Check if buffer ends with a prefix of "</think>" (e.g. "<", "</", "</th")
                    let max_prefix_len = Self::CLOSE_TAG.len() - 1;
                    let mut split_idx = self.buffer.len();

                    for prefix_len in (1..=max_prefix_len).rev() {
                        if self.buffer.len() >= prefix_len {
                            let potential_prefix = &self.buffer[self.buffer.len() - prefix_len..];
                            if Self::CLOSE_TAG.starts_with(potential_prefix) {
                                split_idx = self.buffer.len() - prefix_len;
                                break;
                            }
                        }
                    }

                    if split_idx > 0 {
                        let to_emit = self.buffer[..split_idx].to_string();
                        events.push(ThinkEvent::ThinkChunk(to_emit));
                        self.buffer = self.buffer[split_idx..].to_string();
                    }
                }
            }
        }

        events
    }

    /// Flushes any remaining buffer at the end of the stream.
    pub fn flush(&mut self) -> Vec<ThinkEvent> {
        let mut events = Vec::new();
        if !self.buffer.is_empty() {
            match self.state {
                ParserState::InThink => {
                    events.push(ThinkEvent::ThinkChunk(self.buffer.clone()));
                }
                ParserState::InAnswer => {
                    events.push(ThinkEvent::AnswerChunk(self.buffer.clone()));
                }
            }
            self.buffer.clear();
        }
        events
    }

    pub fn is_in_think(&self) -> bool {
        self.state == ParserState::InThink
    }
}
