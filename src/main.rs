use std::fs;
use std::io;
use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Corner, Direction, Layout},
	style::{Color, Modifier, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, List, Paragraph, Wrap, Clear},
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

	// for command in commands.commands{
	// 	print!("{}", command.output);
	// }

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
						commands.commands[i].output.iter()
							.map(|line| {
								Spans::from(Span::raw(line))
							})
							.collect()
					}
					None => vec![Spans::from(Span::raw("Please select a process"))]
			};

			let block = Paragraph::new(output)
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

	Ok(())
}
