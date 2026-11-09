use a2a::*;
use a2a_server::{AgentExecutor, ExecutorContext};
use apex_core::models::{ChatMessage, ThinkingDepth};
use apex_core::think_parser::ThinkParser;
use apex_web::GroundingEngine;
use async_trait::async_trait;
use chrono::Utc;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::sync::Arc;

use crate::ollama::OllamaClient;

pub struct ApexAgentExecutor {
    ollama: Arc<OllamaClient>,
    grounding: Arc<GroundingEngine>,
}

impl Default for ApexAgentExecutor {
    fn default() -> Self {
        Self::new(
            Arc::new(OllamaClient::default()),
            Arc::new(GroundingEngine::default()),
        )
    }
}

impl ApexAgentExecutor {
    pub fn new(ollama: Arc<OllamaClient>, grounding: Arc<GroundingEngine>) -> Self {
        Self { ollama, grounding }
    }
}

#[async_trait]
impl AgentExecutor for ApexAgentExecutor {
    fn execute(
        &self,
        ctx: ExecutorContext,
    ) -> BoxStream<'static, Result<StreamResponse, A2AError>> {
        let (task_id, context_id) = ctx.task_info();
        let ollama = self.ollama.clone();
        let grounding = self.grounding.clone();

        let query = ctx
            .message
            .as_ref()
            .and_then(|m| m.parts.first())
            .and_then(|p| match &p.content {
                PartContent::Text(t) => Some(t.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "Hello from Apex A2A agent".to_string());

        let s = async_stream::stream! {
            // 1. Emit Working status
            yield Ok(StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                task_id: task_id.clone(),
                context_id: context_id.clone(),
                status: TaskStatus {
                    state: TaskState::Working,
                    message: None,
                    timestamp: Some(Utc::now()),
                },
                metadata: None,
            }));

            // 2. Perform grounding if needed
            let mut prompt_context = String::new();
            if let Ok(res) = grounding.research(&query, 2).await {
                if !res.context_prompt.is_empty() {
                    prompt_context = res.context_prompt;
                }
            }

            let mut messages = Vec::new();
            if !prompt_context.is_empty() {
                messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: prompt_context,
                });
            }
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: query.clone(),
            });

            // 3. Stream from Ollama
            match ollama.stream_chat(messages, ThinkingDepth::Standard).await {
                Ok(mut stream) => {
                    let mut parser = ThinkParser::new();
                    let mut final_answer = String::new();

                    while let Some(chunk_res) = stream.next().await {
                        if let Ok(chunk) = chunk_res {
                            let events = parser.feed(&chunk);
                            for ev in events {
                                match ev {
                                    apex_core::think_parser::ThinkEvent::ThinkChunk(_) => {
                                        // Tokens streamed internally
                                    }
                                    apex_core::think_parser::ThinkEvent::ThinkEnd => {}
                                    apex_core::think_parser::ThinkEvent::AnswerChunk(ans) => {
                                        final_answer.push_str(&ans);
                                    }
                                }
                            }
                        }
                    }

                    for ev in parser.flush() {
                        if let apex_core::think_parser::ThinkEvent::AnswerChunk(ans) = ev {
                            final_answer.push_str(&ans);
                        }
                    }

                    // Emit Agent response message
                    yield Ok(StreamResponse::Message(Message::new(
                        Role::Agent,
                        vec![Part::text(final_answer)],
                    )));

                    // 4. Emit Completed status
                    yield Ok(StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                        task_id: task_id.clone(),
                        context_id: context_id.clone(),
                        status: TaskStatus {
                            state: TaskState::Completed,
                            message: None,
                            timestamp: Some(Utc::now()),
                        },
                        metadata: None,
                    }));
                }
                Err(err_msg) => {
                    // Fallback response for offline or testing mode
                    let fallback = format!("ApexPhi-14B processed request: '{}'. (Engine notice: {})", query, err_msg);
                    yield Ok(StreamResponse::Message(Message::new(
                        Role::Agent,
                        vec![Part::text(fallback)],
                    )));

                    yield Ok(StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                        task_id: task_id.clone(),
                        context_id: context_id.clone(),
                        status: TaskStatus {
                            state: TaskState::Completed,
                            message: None,
                            timestamp: Some(Utc::now()),
                        },
                        metadata: None,
                    }));
                }
            }
        };

        Box::pin(s)
    }

    fn cancel(&self, ctx: ExecutorContext) -> BoxStream<'static, Result<StreamResponse, A2AError>> {
        let (task_id, context_id) = ctx.task_info();
        let s = async_stream::stream! {
            yield Ok(StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                task_id,
                context_id,
                status: TaskStatus {
                    state: TaskState::Canceled,
                    message: None,
                    timestamp: Some(Utc::now()),
                },
                metadata: None,
            }));
        };
        Box::pin(s)
    }
}
