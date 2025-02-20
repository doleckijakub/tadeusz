use std::process::Command;

fn main() {
	let status = Command::new("ollama")
		.arg("create")
		.arg("tadeusz")
		.arg("-f")
		.arg("Modelfile")
		.status()
		.expect("Failed to execute ollama command");

	if !status.success() {
		panic!("ollama create command failed with status: {:?}", status);
	}
}

