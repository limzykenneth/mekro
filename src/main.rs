use std::process::Command;
use serde_json;
use std::fs;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Terminal
};

mod configuration;
use configuration::configuration::Configuration;

fn main() -> Result<(), io::Error> {
	let mut child = Command::new("ls")
		.spawn()
		.expect("Failed to execute command");

	let ecode = child.wait()
		.expect("Failed to wait");

	assert!(ecode.success());

	let contents = fs::read_to_string("./example.json")
		.expect("Something went wrong reading the file");

	let values = parse_configuration(&contents);

	println!("{:?}", values);
	let stdout = io::stdout();
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	loop{
		terminal.draw(|f| {
			let items: Vec<ListItem> = values.iter()
				.map(|v| {
					ListItem::new(v.command)
						.style(Style::default())
				})
				.collect();

			let list = List::new(items)
				.block(Block::default()
					.title("Micro Manage")
					.borders(Borders::ALL))
				.style(Style::default()
					.fg(Color::Black))
				.highlight_style(Style::default()
					.bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD))
				.highlight_symbol(">>");

			let size = f.size();
			f.render_widget(list, size);
		})?;
	}
}

fn parse_configuration(data: &str) -> Vec<Configuration> {
	serde_json::from_str(data).unwrap()
}