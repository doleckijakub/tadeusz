use std::io::{self, BufRead, Write};

use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{Effort, ReasoningConfig, Role},
};

mod ansi;
mod error;
mod tools;

use ansi::*;
use error::{Error, Result};

const MAX_TOOL_ROUNDS: usize = 5;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable not set");

    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/doleckijakub/tadeusz")
        .x_title("Tadeusz")
        .build()?;

    let available_tools = tools::registry::all_tools()?;

    // println!(
    //     "available_tools = {:?}",
    //     available_tools
    //         .iter()
    //         .map(|t| t.function.name.to_string())
    //         .collect::<Vec<_>>()
    // );

    let mut messages = vec![Message::new(
        Role::System,
        "\
        You are Tadeusz, a helpful British alpaca butler.\n\
        \n\
        Always respond in natural language, no tables, emojis or markdown.\n\
        \n\
        You may use tools, but follow these rules:\n\
         - Use WebSearch only when necessary\n\
         - After 2-5 searches, you MUST answer\n\
         - Always use retrieved information - either by answering partially or reasoning about it - before searching again\n\
         - Prefer reasoning over searching\n\
        \n\
         When you have enough information, stop using tools and answer clearly\
        ",
    )];

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();

    loop {
        print!(" {ANSI_GREEN}=>{ANSI_RESET} ");
        stdout.flush()?;

        let mut prompt = String::new();
        stdin.read_line(&mut prompt)?;

        let prompt = prompt.trim_end();

        if prompt.is_empty() {
            continue;
        }

        if prompt == "exit" || prompt == "quit" {
            break;
        }

        messages.push(Message::new(Role::User, prompt));

        let mut tool_rounds = 0;

        loop {
            let force_answer = tool_rounds >= MAX_TOOL_ROUNDS;

            if force_answer {
                messages.push(Message::new(
                    Role::User,
                    "You have used enough tools. Now provide your final answer.",
                ));
            }

            let mut builder = ChatCompletionRequest::builder();
            builder
                .model("openai/gpt-oss-120b:free")
                .messages(messages.clone())
                .reasoning(ReasoningConfig {
                    effort: Some(Effort::Medium),
                    max_tokens: None,
                    exclude: None,
                    enabled: Some(true),
                });
            if !force_answer {
                builder.tools(available_tools.clone()).tool_choice_auto();
            }
            let request = builder.build()?;

            let response = client.chat().create(&request).await?;
            let Some(choice) = response.choices.first() else {
                eprintln!(" {ANSI_RED}(no response){ANSI_RESET}");
                break;
            };

            let content = choice.content().unwrap_or("");
            let reasoning = choice.reasoning().unwrap_or("");
            let tool_calls = choice.tool_calls();

            let assistant_msg = if let Some(calls) = tool_calls {
                Message::assistant_with_tool_calls(content, calls.to_vec())
            } else {
                Message::new(Role::Assistant, content)
            };
            messages.push(assistant_msg);

            if let Some(calls) = tool_calls {
                for tool_call in calls {
                    let tool = tools::registry::create(tool_call)?;
                    let struct_name = tool.debug_name();
                    let debug_str = tool.debug_string();
                    println!("{ANSI_YELLOW}{}{ANSI_RESET}{}", struct_name, &debug_str[struct_name.len()..]);
                    let result = tool.call().await.map_err(Error::ToolExecution)?;
                    println!("{ANSI_MAGENTA}{}{ANSI_RESET}", result);
                    messages.push(Message::tool_response(&tool_call.id, result.trim_end()));
                }
                tool_rounds += 1;
                continue;
            }

            println!("{ANSI_CYAN}REASONING{ANSI_RESET}: {}", reasoning);
            println!("{ANSI_BLUE}RESPONSE{ANSI_RESET}: {}", content);
            break;
        }
    }

    Ok(())
}
