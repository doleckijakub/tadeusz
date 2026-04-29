use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{Stream, StreamExt};
use openrouter_rs::api::chat::{ChatCompletionRequest, Message};
use openrouter_rs::types::completion::FinishReason;
use openrouter_rs::types::stream::StreamEvent;
use openrouter_rs::types::{Effort, ReasoningConfig, Role, Tool as ApiTool};
use openrouter_rs::OpenRouterClient;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

mod error;
mod tools;

const MAX_TOOL_ROUNDS: usize = 5;
const MODEL: &str = "openai/gpt-oss-120b:free";
const SYSTEM_PROMPT: &str = "You are Tadeusz, a helpful British alpaca butler.\n\
    \n\
    Always respond in natural language, no tables, emojis or markdown.\n\
    \n\
    You may use tools, but follow these rules:\n\
     - Use WebSearch only when necessary\n\
     - After 2-5 searches, you MUST answer\n\
     - Always use retrieved information before searching again\n\
     - Prefer reasoning over searching\n\
    \n\
     When you have enough information, stop using tools and answer clearly";

type Sessions = Arc<Mutex<HashMap<Uuid, Vec<Message>>>>;

#[derive(Clone)]
struct AppState {
    client: Arc<OpenRouterClient>,
    tools: Vec<ApiTool>,
    sessions: Sessions,
}

#[derive(Deserialize)]
struct ChatBody {
    session_id: Uuid,
    message: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SsePayload {
    Reasoning { text: String },
    Content { text: String },
    Tool { name: String, args: String },
    ToolResult { text: String },
    Done,
    Error { text: String },
}

impl SsePayload {
    fn to_event(&self) -> Event {
        Event::default().data(serde_json::to_string(self).unwrap_or_default())
    }
}

#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");

    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/doleckijakub/tadeusz")
        .x_title("Tadeusz")
        .build()
        .expect("failed to build OpenRouter client");

    let tools = tools::registry::all_tools().expect("failed to load tools");

    let state = AppState {
        client: Arc::new(client),
        tools,
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/session", post(create_session))
        .route("/api/chat", post(chat))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct SessionResponse {
    session_id: Uuid,
}

async fn create_session(State(state): State<AppState>) -> Json<SessionResponse> {
    let id = Uuid::new_v4();
    state
        .sessions
        .lock()
        .await
        .insert(id, vec![Message::new(Role::System, SYSTEM_PROMPT)]);
    Json(SessionResponse { session_id: id })
}

async fn chat(
    State(state): State<AppState>,
    Json(body): Json<ChatBody>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(chat_stream(state, body)).keep_alive(KeepAlive::default())
}

fn chat_stream(state: AppState, body: ChatBody) -> impl Stream<Item = Result<Event, Infallible>> {
    async_stream::stream! {
        let messages = {
            let mut sessions = state.sessions.lock().await;
            let Some(msgs) = sessions.get_mut(&body.session_id) else {
                yield Ok(SsePayload::Error { text: "unknown session".into() }.to_event());
                return;
            };
            msgs.push(Message::new(Role::User, body.message.as_str()));
            msgs.clone()
        };

        let mut messages = messages;
        let mut tool_rounds = 0;

        'outer: loop {
            let force_answer = tool_rounds >= MAX_TOOL_ROUNDS;
            if force_answer {
                messages.push(Message::new(
                    Role::User,
                    "You have used enough tools. Now provide your final answer.",
                ));
            }

            let mut builder = ChatCompletionRequest::builder();
            builder
                .model(MODEL)
                .messages(messages.clone())
                .reasoning(ReasoningConfig {
                    effort: Some(Effort::Medium),
                    max_tokens: None,
                    exclude: None,
                    enabled: Some(true),
                });
            if !force_answer {
                builder.tools(state.tools.clone()).tool_choice_auto();
            }

            let request = match builder.build() {
                Ok(r) => r,
                Err(e) => {
                    yield Ok(SsePayload::Error { text: e.to_string() }.to_event());
                    return;
                }
            };

            let mut stream = match state.client.chat().stream_tool_aware(&request).await {
                Ok(s) => s,
                Err(e) => {
                    yield Ok(SsePayload::Error { text: e.to_string() }.to_event());
                    return;
                }
            };

            let mut content_buf = String::new();

            while let Some(event) = stream.next().await {
                match event {
                    StreamEvent::ContentDelta(text) => {
                        content_buf.push_str(&text);
                        yield Ok(SsePayload::Content { text }.to_event());
                    }
                    StreamEvent::ReasoningDelta(text) => {
                        yield Ok(SsePayload::Reasoning { text }.to_event());
                    }
                    StreamEvent::Done { tool_calls, finish_reason, .. } => {
                        if matches!(finish_reason, Some(FinishReason::ToolCalls))
                            && !tool_calls.is_empty()
                        {
                            messages.push(Message::assistant_with_tool_calls(
                                content_buf.as_str(),
                                tool_calls.clone(),
                            ));
                            content_buf.clear();

                            for tc in &tool_calls {
                                yield Ok(SsePayload::Tool {
                                    name: tc.name().to_string(),
                                    args: tc.arguments_json().to_string(),
                                }
                                .to_event());

                                let tool = match tools::registry::create(tc) {
                                    Ok(t) => t,
                                    Err(e) => {
                                        yield Ok(SsePayload::Error { text: e.to_string() }
                                            .to_event());
                                        return;
                                    }
                                };

                                let result = match tool.call().await {
                                    Ok(r) => r,
                                    Err(e) => {
                                        yield Ok(SsePayload::Error { text: e }.to_event());
                                        return;
                                    }
                                };

                                yield Ok(SsePayload::ToolResult { text: result.clone() }
                                    .to_event());
                                messages.push(Message::tool_response(tc.id(), result.trim_end()));
                            }

                            tool_rounds += 1;
                        } else {
                            messages.push(Message::new(Role::Assistant, content_buf.as_str()));
                            let mut sessions = state.sessions.lock().await;
                            if let Some(s) = sessions.get_mut(&body.session_id) {
                                *s = messages;
                            }
                            break 'outer;
                        }
                    }
                    StreamEvent::Error(e) => {
                        yield Ok(SsePayload::Error { text: e.to_string() }.to_event());
                        return;
                    }
                    _ => {}
                }
            }
        }

        yield Ok(SsePayload::Done.to_event());
    }
}
