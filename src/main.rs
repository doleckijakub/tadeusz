use std::io::{self, BufRead, Write};

use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

mod tools;

use tools::ToolType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable not set");

    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/doleckijakub/tadeusz")
        .x_title("Tadeusz")
        .build()?;

    let available_tools = {
        let web_search_tool = tools::tool::<tools::web_search::WebSearch>()?;
        let web_fetch_tool = tools::tool::<tools::web_fetch::WebFetch>()?;

        vec![web_search_tool, web_fetch_tool]
    };

    let mut messages = vec![Message::new(
        Role::System,
        "\
        You are Tadeusz, a helpful british alpaca butler.\n\
        Always respond in natural language, no tables, emojis or markdown.\n\
        Use relevant tools whenever appropriate.\n\
        ",
    )];

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();

    loop {
        print!(" => ");
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

        loop {
            let request = ChatCompletionRequest::builder()
                .model("openai/gpt-oss-120b:free")
                .messages(messages.clone())
                .tools(available_tools.clone())
                .tool_choice_auto()
                .build()?;

            let response = client.chat().create(&request).await?;
            let Some(choice) = response.choices.first() else {
                eprintln!(" <= (no response)");
                break;
            };

            let content = choice.content().unwrap_or("");
            let tool_calls = choice.tool_calls();

            let assistant_msg = if let Some(calls) = tool_calls {
                Message::assistant_with_tool_calls(content, calls.to_vec())
            } else {
                Message::new(Role::Assistant, content)
            };
            messages.push(assistant_msg);

            if let Some(calls) = tool_calls {
                for tool_call in calls {
                    let result = match tool_call.function.name.as_str() {
                        "web_search" => {
                            let tool: tools::web_search::WebSearch =
                                serde_json::from_str(&tool_call.function.arguments)?;
                            println!("[*] {:?}", tool);
                            tool.execute().await?
                        }
                        "web_fetch" => {
                            let tool: tools::web_fetch::WebFetch =
                                serde_json::from_str(&tool_call.function.arguments)?;
                            println!("[*] {:?}", tool);
                            tool.execute().await?
                        }
                        _ => {
                            unimplemented!("Handling of unknown tools")
                        }
                    };

                    messages.push(Message::tool_response(&tool_call.id, result.trim_end()));
                }
                continue;
            }

            println!(" <= {}", content);
            break;
        }
    }

    Ok(())
}
