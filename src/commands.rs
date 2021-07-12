pub mod commands{
	use serde::{Serialize, Deserialize};
	use tui::{
		widgets::{ListState, ListItem},
		style::{Style},
	};
	use serde_json;
	use std::process::{self, Child, ChildStdout};

	#[derive(Serialize, Deserialize, Debug)]
	pub struct Command<'a>{
		pub command: &'a str,
		pub arguments: Vec<&'a str>,
	}

	impl Command<'_>{
		fn run(&self) -> Child {
			let mut child = process::Command::new(self.command)
				.args(&self.arguments)
				.spawn()
				.expect("Failed to execute command");

			child
		}
	}

	pub struct Commands<'a>{
		pub items: Vec<ListItem<'a>>,
		pub state: ListState,
		pub commands: Vec<Command<'a>>,
		pub processes: Vec<Child>,
		pub stdouts: Vec<ChildStdout>
	}

	impl Commands<'_>{
		pub fn new(config: &str) -> Commands {
			let values = parse_configuration(config);
			let items: Vec<ListItem> = values.iter()
				.map(|v| {
					ListItem::new(v.command)
						.style(Style::default())
				})
				.collect();

			Commands {
				items,
				state: ListState::default(),
				commands: values,
				processes: vec!(),
				stdouts: vec!()
			}
		}

		pub fn run(&mut self){
			for command in &self.commands {
				let mut child = command.run();
				// self.stdouts.push(child.stdout.take().unwrap());
				self.processes.push(child);
			}
		}

		pub fn next(&mut self) {
			let i = match self.state.selected() {
				Some(i) => {
					if i >= self.items.len() - 1 {
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
						self.items.len() - 1
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

	fn parse_configuration(data: &str) -> Vec<Command> {
		serde_json::from_str(data).unwrap()
	}
}