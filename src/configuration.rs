pub mod configuration{
	use serde::{Serialize, Deserialize};
	use serde_json;

	#[derive(Serialize, Deserialize, Debug)]
	pub struct Configuration<'a>{
		pub name: &'a str,
		pub command: &'a str,
		pub arguments: Vec<&'a str>
	}

	pub fn parse_configuration(data: &str) -> Vec<Configuration> {
		serde_json::from_str(data).unwrap()
	}
}