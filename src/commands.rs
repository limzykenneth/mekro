pub mod commands{
	use tui::{
		widgets::{ListState, ListItem},
		style::{Style},
	};
	use tokio::{
		process::{
			Command as Cmd,
			Child
		},
		io::{BufReader, AsyncBufReadExt},
		sync::{
			mpsc::{channel as mpscChannel, Sender, Receiver},
			Mutex as TokioMutex
		}
	};
	use std::{
		process::Stdio,
		sync::{Arc, Mutex}
	};
	use nix::{
		sys::signal::{
			Signal,
			killpg
		},
		unistd::Pid
	};
	use crate::configuration::configuration::{
		Configuration,
		parse_configuration
	};

	#[derive(Debug)]
	pub struct Command<'a>{
		pub child_process: Option<Arc<Mutex<Child>>>,
		command: &'a str,
		arguments: Vec<&'a str>,
		pub output: Arc<Mutex<Vec<String>>>,
		pub tx: Arc<Sender<String>>,
		pub rx: Arc<TokioMutex<Receiver<String>>>
	}

	impl Command<'_>{
		fn new<'a>(config: &Configuration<'a>) -> Command<'a>{
			let (tx, rx) = mpscChannel(100);

			Command {
				child_process: None,
				command: config.command,
				arguments: config.arguments.clone(),
				output: Arc::new(Mutex::new(vec!())),
				tx: Arc::new(tx),
				rx: Arc::new(TokioMutex::new(rx))
			}
		}

		pub async fn run(&mut self){
			// Clear output vector
			self.output.lock().unwrap().clear();

			let mut cmd = Cmd::new(self.command);
			cmd.stdout(Stdio::piped());
			cmd.stderr(Stdio::piped());
			cmd.args(&self.arguments);
			unsafe{
				cmd.pre_exec(move ||{
					let pid = nix::unistd::getpid();
					nix::unistd::setpgid(pid, pid)
						.expect("Failed to set process group ID");
					Result::Ok(())
				});
			}

			let mut child = cmd
				.spawn()
				.expect("Failed to execute command");

			match child.stdout.take() {
				Some(stdout) => {
					let mut stdout = BufReader::new(stdout).lines();
					let tx = self.tx.clone();
					tokio::spawn(async move {
						while let Some(line) = stdout.next_line().await.unwrap() {
							tx.send(line).await.unwrap();
						}
					});
				}
				None => ()
			};

			match child.stderr.take() {
				Some(stderr) => {
					let mut stderr = BufReader::new(stderr).lines();
					let tx = self.tx.clone();
					tokio::spawn(async move {
						while let Some(line) = stderr.next_line().await.unwrap() {
							tx.send(line).await.unwrap();
						}
					});
				}
				None => ()
			};

			self.child_process = Some(Arc::new(Mutex::new(child)));

			let rx = self.rx.clone();
			let output = self.output.clone();
			tokio::spawn(async move {
				let mut rx = rx.lock().await;
				while let Some(message) = rx.recv().await {
					let mut output = output.lock().unwrap();
					output.push(message);
				}
			});
		}

		pub async fn kill(&self){
			let child = self.child_process.as_ref().unwrap().lock().unwrap();

			killpg(Pid::from_raw(child.id().unwrap() as i32), Signal::SIGINT)
				.expect("Failed to kill process group");
		}
	}

	pub struct Commands<'a>{
		pub state: ListState,
		pub commands: Vec<Command<'a>>,
		pub items: Vec<ListItem<'a>>
	}

	impl Commands<'_>{
		pub fn new(config: &str) -> Commands {
			let values: Vec<Configuration> = parse_configuration(config);
			let commands: Vec<Command> = values.iter()
				.map(|v| {
					Command::new(v)
				})
				.collect();
			let items: Vec<ListItem> = commands.iter()
				.map(|command| {
					ListItem::new(command.command)
						.style(Style::default())
				})
				.collect();

			Commands {
				state: ListState::default(),
				commands: commands,
				items: items
			}
		}

		pub async fn run(&mut self){
			for command in &mut self.commands {
				command.run().await;
			}
		}

		pub async fn kill(&mut self){
			for command in &self.commands {
				command.kill().await;
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