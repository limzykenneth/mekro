pub mod commands{
	use tui::widgets::{ListState, ListItem};
	use tokio::{
		io::{BufReader, AsyncBufReadExt},
		sync::{
			mpsc::{channel as mpscChannel, Sender, Receiver},
			Mutex as TokioMutex
		},
		fs::File
	};
	use std::{
		sync::{Arc, Mutex},
		path::Path,
		ffi::CString,
		os::unix::io::{FromRawFd, IntoRawFd}
	};
	use nix::{
		sys::{
			signal::{
				Signal,
				killpg
			},
			stat
		},
		unistd::{
			Pid,
			fork,
			ForkResult,
			dup2
		},
		fcntl::{open, OFlag},
		pty::{
			grantpt,
			posix_openpt,
			ptsname,
			unlockpt,
			Winsize
		},
		ioctl_none_bad,
		ioctl_write_ptr_bad
	};
	use libc::{
		STDIN_FILENO,
		STDOUT_FILENO,
		STDERR_FILENO,
		TIOCSCTTY,
		TIOCSWINSZ
	};
	use terminal_size::{Width, Height, terminal_size};

	use crate::configuration::configuration::{
		Configuration,
		parse_configuration
	};

	// nix macro that generates an ioctl call to set window size of pty:
	ioctl_write_ptr_bad!(set_window_size, TIOCSWINSZ, Winsize);
	// request to "Make the given terminal the controlling terminal of the calling process"
	ioctl_none_bad!(set_controlling_terminal, TIOCSCTTY);

	#[derive(Debug)]
	pub struct Command<'a>{
		name: &'a str,
		command: &'a str,
		arguments: Vec<&'a str>,
		child_pid: Option<Pid>,
		tx: Arc<Sender<String>>,
		rx: Arc<TokioMutex<Receiver<String>>>,
		pub output: Arc<Mutex<Vec<String>>>
	}

	impl Command<'_>{
		fn new<'a>(config: &Configuration<'a>) -> Command<'a>{
			let (tx, rx) = mpscChannel(100);

			Command {
				name: config.name,
				command: config.command,
				arguments: config.arguments.clone(),
				child_pid: None,
				tx: Arc::new(tx),
				rx: Arc::new(TokioMutex::new(rx)),
				output: Arc::new(Mutex::new(vec![])),
			}
		}

		pub async fn run(&mut self){
			// Clear output vector
			self.output.lock().unwrap().clear();

			// Create new pty
			let master_fd = posix_openpt(OFlag::O_RDWR).unwrap();
			grantpt(&master_fd).unwrap();
			unlockpt(&master_fd).unwrap();

			let slave_name = unsafe { ptsname(&master_fd) }.unwrap();

			unsafe {
				self.child_pid = Some(match fork() {
					Ok(ForkResult::Child) => {
						// Open slave end for pseudoterminal
						let slave_fd = open(Path::new(&slave_name), OFlag::O_RDWR, stat::Mode::empty()).unwrap();

						let size = terminal_size();
						if let Some((Width(w), Height(h))) = size {
							let winsize = Winsize {
								ws_row: h,
								ws_col: (w as f32 * 0.8).floor() as u16 - 2,
								ws_xpixel: 0,
								ws_ypixel: 0,
							};
							set_window_size(slave_fd.into_raw_fd(), &winsize).unwrap();
						}

						// assign stdin, stdout, stderr to the tty
						dup2(slave_fd, STDIN_FILENO).unwrap();
						dup2(slave_fd, STDOUT_FILENO).unwrap();
						dup2(slave_fd, STDERR_FILENO).unwrap();
						// Become session leader
						nix::unistd::setsid().unwrap();

						set_controlling_terminal(slave_fd).unwrap();

						let mut args: Vec<CString> = vec![CString::new(self.command).unwrap()];
						args.append(&mut self.arguments.iter()
							.map(|argument| {
								CString::new(argument.to_owned()).unwrap()
							})
							.collect()
						);
						nix::unistd::execvp(
							&CString::new(self.command).unwrap(),
							&args
						).unwrap();

						// This path shouldn't be executed.
						std::process::exit(-1);
					}
					Ok(ForkResult::Parent { child }) => child,
					Err(e) => panic!("{}", e),
				});
			}

			let master_file = unsafe {
				File::from_raw_fd(master_fd.into_raw_fd())
			};
			let mut stdout = BufReader::new(master_file).lines();

			// MPSC sender
			let tx = self.tx.clone();
			tokio::spawn(async move {
				while let Ok(Some(line)) = stdout.next_line().await {
					tx.send(line).await.unwrap();
				}
			});

			// MPSC receiver
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
			// NOTE: Look into macOS behaviour (leaving orphan processes)
			// https://superuser.com/questions/99789/when-a-python-process-is-killed-on-os-x-why-doesnt-it-kill-the-child-processes
			killpg(self.child_pid.unwrap(), Signal::SIGINT).unwrap();
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
				.map(|v| Command::new(v))
				.collect();
			let items: Vec<ListItem> = commands.iter()
				.map(|command| ListItem::new(command.name))
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

		pub async fn kill(&self){
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