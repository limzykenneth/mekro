use std::process::Command;
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
use std::time::Duration;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod configuration;
use configuration::configuration::Configurations;

fn main() -> Result<(), io::Error> {
	let mut child = Command::new("ls")
		.spawn()
		.expect("Failed to execute command");

	let ecode = child.wait()
		.expect("Failed to wait");

	assert!(ecode.success());

	let contents = fs::read_to_string("./example.json")
		.expect("Something went wrong reading the file");

	let stdout = io::stdout();
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut configurations = Configurations::new(&contents);

	enable_raw_mode().unwrap();
	loop{
		if poll(Duration::from_millis(100)).unwrap() {
			// It's guaranteed that `read` wont block, because `poll` returned
			// `Ok(true)`.
			match read().unwrap(){
				Event::Key(KeyEvent {
					code: KeyCode::Char('c'),
					modifiers: KeyModifiers::CONTROL
				}) => break,
				Event::Key(KeyEvent {
					code: KeyCode::Down,
					modifiers: KeyModifiers::NONE
				}) => {
					// println!("Down");
					configurations.next();
				},
				Event::Key(KeyEvent {
					code: KeyCode::Up,
					modifiers: KeyModifiers::NONE
				}) => {
					// println!("Up");
					configurations.previous();
				},
				_ => ()
			};
		} else {
			// Timeout expired, no `Event` is available
		}

		terminal.draw(|f| {
			let items = configurations.items.clone();

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
			f.render_stateful_widget(list, size, &mut configurations.state);
		})?;
	}

	disable_raw_mode().unwrap();
	Ok(())
}
