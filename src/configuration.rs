pub mod configuration{
	use serde::{Serialize, Deserialize};
	use tui::widgets::ListState;
	use serde_json;

	#[derive(Serialize, Deserialize, Debug)]
	pub struct Configuration<'a>{
		pub command: &'a str,
		pub arguments: Vec<&'a str>
	}

	pub struct Configurations<'a>{
		items: Vec<Configuration<'a>>,
		state: ListState
	}

	// impl Configurations{
	// 	fn new(config: &str) -> Configurations{
	// 		let parse_configuration(config)
	// 	}
	// }

	// fn parse_configuration(data: &str) -> Vec<Configuration> {
	// 	serde_json::from_str(data).unwrap()
	// }
}