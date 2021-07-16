use std::fs;
use std::io;
use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	text::{Span, Spans, Text},
	widgets::{Block, Borders, List, Paragraph, Wrap},
	Terminal
};
use std::time::Duration;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;

mod configuration;
mod commands;
use commands::commands::Commands;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
	let contents = fs::read_to_string("./example.json")
		.expect("Something went wrong reading the file");

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

			let output: Vec<Spans> = match commands.state.selected() {
					Some(i) => {
						commands.commands[i].output.lock().unwrap().iter()
							.map(|line| {
								Spans::from(Span::raw(line.clone()))
							})
							.collect()
					}
					None => vec![Spans::from(Span::raw("Please select a process"))]
			};

			let block = Paragraph::new(Text::from(output))
				.block(Block::default()
					.title("stdout")
					.borders(Borders::ALL)
				)
				.style(Style::default()
					.fg(Color::Black))
				.wrap(Wrap {trim: true});

			f.render_widget(block, chunks[1]);
		})?;
	}

	commands.kill().await;

	Ok(())
}
