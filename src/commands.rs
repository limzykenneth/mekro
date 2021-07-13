pub mod commands{
	use tui::{
		widgets::{ListState, ListItem},
		style::{Style},
	};
	use tokio::process::{Command as Cmd, Child};
	use crate::configuration::configuration::{
		Configuration,
		parse_configuration
	};

	pub struct Command<'a>{
		pub item: ListItem<'a>,
		pub child_process: Option<Child>,
		command: &'a str,
		arguments: Vec<&'a str>
	}

	impl Command<'_>{
		fn new<'a>(config: &Configuration<'a>) -> Command<'a>{
			Command {
				item: ListItem::new(config.command)
					.style(Style::default()),
				child_process: None,
				command: config.command,
				arguments: config.arguments.clone()
			}
		}

		fn run(&mut self){
			let child = Cmd::new(self.command)
				.args(&self.arguments)
				.spawn()
				.expect("Failed to execute command");

			self.child_process = Some(child);
		}
	}

	pub struct Commands<'a>{
		pub state: ListState,
		pub commands: Vec<Command<'a>>
	}

	impl Commands<'_>{
		pub fn new(config: &str) -> Commands {
			let values: Vec<Configuration> = parse_configuration(config);
			let commands: Vec<Command> = values.iter()
				.map(|v| {
					Command::new(v)
				})
				.collect();

			Commands {
				state: ListState::default(),
				commands: commands
			}
		}

		pub fn run(&mut self){
			for command in &mut self.commands {
				command.run();
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