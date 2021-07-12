pub mod configuration{
	use serde::{Serialize, Deserialize};
	use tui::{
		widgets::{ListState, ListItem},
		style::{Style},
	};
	use serde_json;

	#[derive(Serialize, Deserialize, Debug)]
	pub struct Configuration<'a>{
		pub command: &'a str,
		pub arguments: Vec<&'a str>
	}

	pub struct Configurations<'a>{
		pub items: Vec<ListItem<'a>>,
		pub state: ListState
	}

	impl Configurations<'_>{
		pub fn new(config: &str) -> Configurations{
			let values = parse_configuration(config);
			let items: Vec<ListItem> = values.iter()
				.map(|v| {
					ListItem::new(v.command)
						.style(Style::default())
				})
				.collect();

			Configurations{
				items,
				state: ListState::default()
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

	fn parse_configuration(data: &str) -> Vec<Configuration> {
		serde_json::from_str(data).unwrap()
	}
}