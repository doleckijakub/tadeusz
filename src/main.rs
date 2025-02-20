use reqwest;
use serde_json::Value;
use std::io::{self, Write};
use tokio_stream::{Stream, StreamExt};
use bytes::Bytes;

// #[derive(Debug)]
// #[allow(dead_code)]
// struct Color {
//     r: u8,
//     g: u8,
//     b: u8,
// }
// 
// impl From<String> for Color {
//     fn from(str: String) -> Self {
//         match str.as_str() {
//             "off" => Self { r: 0, g: 0, b: 0 },
//             _ => {
//                 if str.len() == 6 {
//                     Self {
//                         r: u8::from_str_radix(&str[0..2], 16).unwrap_or(0),
//                         g: u8::from_str_radix(&str[2..4], 16).unwrap_or(0),
//                         b: u8::from_str_radix(&str[4..6], 16).unwrap_or(0),
//                     }
//                 } else {
//                     Self { r: 0, g: 0, b: 0 }
//                 }
//             }
//         }
//     }
// }
// 
// impl Color {
//     fn to_string(&self) -> String {
//         format!("{:02x}{:02x}{:02x}", self.r, self.g, self.g)
//     }
// }

#[derive(Debug)]
enum TokenType {
    Str(String),
    Symbol(char),
    Identifier(String),
}

#[derive(Debug)]
struct Token {
    r#type: TokenType,
    start: usize,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match &self.r#type {
            TokenType::Str(content) => content.to_string(),
            TokenType::Symbol(c) => format!("{}", c),
            TokenType::Identifier(content) => content.to_string(),
        })
    }
}

// #[derive(Debug)]
// #[allow(dead_code)]
// enum Command {
//     Say(String),
//     Whisper(String),
//     SetLed(String, Color),
//     PlayMusic(String),
//     PlaySound(String),
//     PlaySoundEffect(String),
//     SetAlarm(String, String),
//     DuckDuckGo(String),
// }

#[derive(Clone)]
struct Tadeusz {
    input_buffer: String,
    last_token_start: usize,
    done: bool,
}

impl Tadeusz {
    fn new() -> Self {
        Self {
            input_buffer: String::new(),
            last_token_start: 0,
            done: false,
        }
    }

    fn reset(&mut self) {
        let new_self = Self::new();

        self.input_buffer = new_self.input_buffer;
        self.last_token_start = new_self.last_token_start;
        self.done = new_self.done;
    }

    fn stop(&mut self) {
        self.done = true;
        // TODO: kill Tadeusz and start a new one with same input, until it works xd
        self.reset();
    }

    async fn handle_message_chunk(&mut self, stream: &mut (impl Stream<Item = reqwest::Result<Bytes>> + Unpin)) {
        while let Some(chunk) = stream.next().await {
            if let Ok(bytes) = chunk {
                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                    let data: Value = serde_json::from_str(&text).expect("Could not parse ollama as a JSON response");

                    let response = data["response"].as_str().unwrap();
                    let done = data["done"].as_bool().unwrap();

                    if done {
                        self.done = true;
                        break;
                    } else {
                        self.input_buffer.push_str(response);
                    }

                    self.handle_new_tokens();
                }
            }
        }
    }

    fn handle_new_tokens(&mut self) {
        if self.done {
            eprintln!("I'm done here");
            return;
        }

        let tokens = self.tokenize(self.last_token_start);

        let types: Vec<_> = tokens.iter().map(|t| &t.r#type).collect();

        match types.as_slice() {
            [
                TokenType::Symbol('<'),
                TokenType::Identifier(name),
                TokenType::Symbol('>')
            ] => {
                if name != "Tadeusz" {
                    eprintln!("It seems like someone else needs to speek now ({})", name);
                    self.stop();    
                }

                self.last_token_start = tokens.last().unwrap().start + 1;
            },
            [
                TokenType::Identifier(func),
                TokenType::Symbol('('),
                TokenType::Str(arg0),
                TokenType::Symbol(')')
            ] => {
                println!("$ {}({:?})", func, arg0);
                self.last_token_start = tokens.last().unwrap().start + 1;
            },
            [
                TokenType::Identifier(func),
                TokenType::Symbol('('),
                TokenType::Str(arg0),
                TokenType::Symbol(','),
                TokenType::Str(arg1),
                TokenType::Symbol(')')
            ] => {
                println!("$ {}({:?}, {:?})", func, arg0, arg1);
                self.last_token_start = tokens.last().unwrap().start + 1;
            },
            _ => {
                // TODO: ponder what to do here
                // eprintln!("{}", &self.input_buffer[self.last_token_start..].trim());
                // for token in tokens {
                //     eprint!("{}", token);
                //     io::stdout().flush().unwrap();
                // }
                // eprintln!();
            }
        }
    }

    fn tokenize(&self, start: usize) -> Vec<Token> {
        let mut input = self.input_buffer.chars().enumerate().skip(start).peekable();
        let mut tokens = Vec::new();

        while let Some(&(idx, ch)) = input.peek() {
            match ch {
                c if c.is_whitespace() => { input.next(); },
                c if c.is_alphabetic() => {
                    let mut token = String::new();
                    let start_idx = idx;
                    while let Some(&(_, ch)) = input.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            token.push(ch);
                            input.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token { r#type: TokenType::Identifier(token), start: start_idx });
                },
                '<' | '>' | '(' | ')' | ',' => {
                    tokens.push(Token { r#type: TokenType::Symbol(ch), start: idx });
                    input.next();
                },
                '"' => {
                    input.next();
                    let mut token = String::new();
                    let start_idx = idx;
                    while let Some(&(_, ch)) = input.peek() {
                        if ch == '\\' {
                            input.next();
                            if let Some(&(_, escaped)) = input.peek() {
                                token.push(escaped);
                                input.next();
                            }
                        } else if ch == '"' {
                            break;
                        } else {
                            token.push(ch);
                            input.next();
                        }
                    }
                    input.next();
                    tokens.push(Token { r#type: TokenType::Str(token), start: start_idx });
                },
                _ => { input.next(); },
            }
        }

        tokens
    }
}

#[tokio::main]
async fn main() {
    let mut tadeusz = Tadeusz::new();

    loop {
        print!(" => ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().eq_ignore_ascii_case("exit") {
            break;
        }

        if input.find('<').unwrap_or(255) > 3 {
            input = format!("<User>\n{input}");
        }

        match send_to_ollama(&input, &mut tadeusz).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error sending input to ollama: {}", e);
            }
        }

        tadeusz.reset();
    }
}

async fn send_to_ollama(prompt: &str, tadeusz: &mut Tadeusz) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "tadeusz",
            "prompt": prompt,
            "stream": true
        }))
        .send()
        .await?;

    let mut stream = res.bytes_stream();
    tadeusz.handle_message_chunk(&mut stream).await;

    Ok(())
}

