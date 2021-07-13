pub mod commands{
	use tui::{
		widgets::{ListState, ListItem},
		style::{Style},
	};
	use tokio::process::{
		Command as Cmd,
		Child,
		ChildStdout
	};
	use tokio::io::{BufReader, AsyncBufReadExt, Lines};
	use std::process::Stdio;
	use crate::configuration::configuration::{
		Configuration,
		parse_configuration
	};

	#[derive(Debug)]
	pub struct Command<'a>{
		pub child_process: Option<Child>,
		pub stdout: Option<Lines<BufReader<ChildStdout>>>,
		command: &'a str,
		arguments: Vec<&'a str>,
		pub output: String
	}

	impl Command<'_>{
		fn new<'a>(config: &Configuration<'a>) -> Command<'a>{
			Command {
				child_process: None,
				stdout: None,
				command: config.command,
				arguments: config.arguments.clone(),
				output: String::from("")
			}
		}

		async fn run(&mut self){
			let mut cmd = Cmd::new(self.command);
			cmd.stdout(Stdio::piped());
			cmd.args(&self.arguments);

			let mut child = cmd
				.spawn()
				.expect("Failed to execute command");

			let stdout = child.stdout.take()
				 .expect("child did not have a handle to stdout");

			let mut stdout = BufReader::new(stdout).lines();
			self.child_process = Some(child);

			// tokio::spawn(async move {
			// 	let status = child.wait().await
			// 		.expect("child process encountered an error");

			// 	println!("child status was: {}", status);
			// });

			while let Some(line) = stdout.next_line().await.unwrap() {
				self.output.push_str(&line);
				self.output.push_str("\n");
			}
		}
	}

	pub struct Commands<'a>{
		pub state: ListState,
		pub commands: Vec<Command<'a>>,
		pub items: Vec<ListItem<'a>>
	}

	impl Commands<'_>{
		pub fn new(config: &str) -> Commands {
			let values: Vec<Configuration> = parse_configuration(config);
			let commands: Vec<Command> = values.iter()
				.map(|v| {
					Command::new(v)
				})
				.collect();
			let items: Vec<ListItem> = commands.iter()
				.map(|command| {
					ListItem::new(command.command)
						.style(Style::default())
				})
				.collect();

			Commands {
				state: ListState::default(),
				commands: commands,
				items: items
			}
		}

		pub async fn run(&mut self){
			for command in &mut self.commands {
				command.run().await;
			}
		}

		pub fn next(&mut self) {
			let i = match self.state.selected() {
				Some(i) => {
					if i >= self.commands.len() - 1 {
						0
					} else {
						i + 1
					}
				}
				None => 0,
			};
			self.state.select(Some(i));
		}

		pub fn previous(&mut self) {
			let i = match self.state.selected() {
				Some(i) => {
					if i == 0 {
						self.commands.len() - 1
					} else {
						i - 1
					}
				}
				None => 0,
			};
			self.state.select(Some(i));
		}

		pub fn unselect(&mut self) {
			self.state.select(None);
		}
	}
}