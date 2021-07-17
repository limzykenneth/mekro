mod configuration;
mod commands;

use std::{
	fs,
	io,
	cmp::max,
	time::Duration,
	process
};
use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	text::{Span, Spans, Text},
	widgets::{Block, Borders, List, Paragraph},
	Terminal
};
use crossterm::{
	event::{
		poll, read, Event, KeyCode, KeyEvent, KeyModifiers
	},
	terminal::{
		disable_raw_mode,
		enable_raw_mode,
		EnterAlternateScreen,
		LeaveAlternateScreen
	},
	execute
};
use textwrap::{
	wrap,
	Options as WrapOptions,
	wrap_algorithms::FirstFit
};
use clap::{Arg, App};
use commands::commands::Commands;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
	let matches = App::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg(Arg::with_name("config")
			.short("c")
			.long("config")
			.value_name("FILE")
			.help("Set path to configuration file")
			.takes_value(true)
		)
		.get_matches();
	let config_path = matches.value_of("config").unwrap_or("./example.json");
	let contents = match fs::read_to_string(config_path) {
		Ok(result) => result,
		Err(e) => {
			if e.kind() == io::ErrorKind::NotFound {
				println!("Config file not found");
				process::exit(1);
			}else{
				println!("Something went wrong reading the config file");
				process::exit(1);
			}
		}
	};

	let mut commands = Commands::new(&contents);
	commands.run().await;

	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen).unwrap();
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	enable_raw_mode().unwrap();

	loop{
		if poll(Duration::from_millis(100)).unwrap() {
			// It's guaranteed that `read` wont block, because `poll` returned
			// `Ok(true)`.
			match read().unwrap(){
				Event::Key(KeyEvent {
					code: KeyCode::Char('c'),
					modifiers: KeyModifiers::CONTROL
				}) => {
					disable_raw_mode().unwrap();
					execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen
                    ).unwrap();
					break;
				},
				Event::Key(KeyEvent {
					code: KeyCode::Esc,
					modifiers: KeyModifiers::NONE
				}) => {
					commands.unselect();
				},

				// Navigate between the different tasks
				Event::Key(KeyEvent {
					code: KeyCode::Down,
					modifiers: KeyModifiers::NONE
				}) => {
					commands.next();
				},
				Event::Key(KeyEvent {
					code: KeyCode::Up,
					modifiers: KeyModifiers::NONE
				}) => {
					commands.previous();
				},

				// Navigate between pages
				Event::Key(KeyEvent {
					code: KeyCode::Left,
					modifiers: KeyModifiers::NONE
				}) => {

				},
				Event::Key(KeyEvent {
					code: KeyCode::Right,
					modifiers: KeyModifiers::NONE
				}) => {

				},

				// Action on the selected task
				// Start
				Event::Key(KeyEvent {
					code: KeyCode::Char('s'),
					modifiers: KeyModifiers::NONE
				}) => {
					match commands.state.selected() {
						Some(i) => {
							commands.commands[i].run().await;
						}
						None => ()
					}
				},
				// Kill
				Event::Key(KeyEvent {
					code: KeyCode::Char('k'),
					modifiers: KeyModifiers::NONE
				}) => {
					match commands.state.selected() {
						Some(i) => {
							commands.commands[i].kill().await;
						}
						None => ()
					}
				},
				// Restart
				Event::Key(KeyEvent {
					code: KeyCode::Char('r'),
					modifiers: KeyModifiers::NONE
				}) => {
					match commands.state.selected() {
						Some(i) => {
							commands.commands[i].kill().await;
							commands.commands[i].run().await;
						}
						None => ()
					}
				},
				_ => ()
			};
		} else {
			// Timeout expired, no `Event` is available
		}

		terminal.draw(|f| {
			let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(f.size());

			let list = List::new(commands.items.clone())
				.block(Block::default()
					.title("Micro Manage")
					.borders(Borders::ALL))
				.style(Style::default()
					.fg(Color::Black))
				.highlight_style(Style::default()
					.bg(Color::LightGreen)
					.add_modifier(Modifier::BOLD))
				.highlight_symbol(">>");

			f.render_stateful_widget(list, chunks[0], &mut commands.state);

			let mut paragraph_height = 0;
			let mut display_text: Vec<Spans> = vec!();
			let command_output: Vec<String>;

			match commands.state.selected() {
				Some(i) => {
					command_output = commands.commands[i].output.lock().unwrap().to_vec();
					command_output.iter()
						.for_each(|line| {
							let options = WrapOptions::new((chunks[1].width-2) as usize)
								.wrap_algorithm(FirstFit);
							let wrapped_lines = wrap(&line, &options);

							paragraph_height += wrapped_lines.len();

							for line in wrapped_lines {
								display_text.push(Spans::from(Span::raw(line)))
							}
						});
				}
				None => {
					display_text = vec![Spans::from(Span::raw("Please select a process"))];
				}
			};


			let scroll_height = max(paragraph_height as i32 - (chunks[1].height-2) as i32, 0) as u16;

			let block = Paragraph::new(Text::from(display_text))
				.scroll((scroll_height, 0))
				.block(Block::default()
					.title("stdout")
					.borders(Borders::ALL)
				)
				.style(Style::default()
					.fg(Color::Black));

			f.render_widget(block, chunks[1]);
		})?;
	}

	commands.kill().await;

	Ok(())
}
