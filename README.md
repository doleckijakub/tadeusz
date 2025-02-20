# Tadeusz - an AI home assistant

Tadeusz is an AI, that automates your home (TODO: finish when the features are actually implemented)

## Features

 - Processes streaming responses from Ollama.
 - Tokenizes input dynamically and interprets function calls.
 - Supports basic command execution such as `say`, `set_led`, `play_music`, and more.
 - Implements a structured approach for handling AI-generated responses.

# Prerequisites

 - Rust (latest stable version)
 - Cargo package manager
 - An instance of Ollama running locally

# Installation

## Clone the repository and navigate to the project directory:

```console
git clone https://github.com/your-repo/tadeusz.git  
cd tadeusz
```

## Install dependencies:

```console
ollama create tadeusz -f Modelfile
cargo build
```

# Running the Application

Ensure that Ollama is running on http://localhost:11434, then start the application:

```console
cargo run 
```

You can then interact with Tadeusz by entering text into the console. To exit, type:

```console
exit
```
