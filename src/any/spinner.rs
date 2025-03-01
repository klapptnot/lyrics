use rand::Rng;
use std::io::Write;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Spinner {
  handle: Option<thread::JoinHandle<()>>, // Stores the thread handle
  channel: Option<Sender<bool>>,          // Stores the comunication channel
  stop_flag: Arc<Mutex<bool>>,            // Shared flag to signal stop
}

impl Spinner {
  pub fn new() -> Self {
    Spinner {
      handle: None,
      channel: None,
      stop_flag: Arc::new(Mutex::new(false)),
    }
  }

  pub fn start(&mut self, hint: String) -> &mut Self {
    let (emtr, recv) = channel::<bool>();

    self.handle = Some(thread::spawn(move || {
      let chars = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
      let mut i: usize = 0;
      // Set cursor invisible
      print!("\x1b[?25l");
      loop {
        let stop_me = match recv.try_recv() {
          Ok(_) | Err(TryRecvError::Disconnected) => true,
          _ => false,
        };
        if stop_me {
          break;
        }
        let mut rng = rand::thread_rng();
        let code: u8 = rng.gen();
        // Delete from cursor to end, print spin, move cursor to start
        print!("\x1b[0K\x1b[38;5;{}m{}\x1b[0m {}\x1b[0G", code, chars[i], hint);
        std::io::stdout().flush().unwrap();
        i += 1;
        if i >= chars.len() {
          i = 0
        }
        thread::sleep(std::time::Duration::from_millis(80 as u64));
      }
      // Go to line start, delete from cursor to end, set cursor visible
      print!("\x1b[0K\x1b[?25h");
    }));
    self.channel = Some(emtr);
    self
  }

  pub fn update(&mut self, hint: String) -> &mut Self {
    self.stop();
    self.start(hint)
  }

  pub fn stop(&mut self) {
    // Set the stop flag
    let mut lock = self.stop_flag.lock().unwrap();
    *lock = true;

    self
      .channel
      .as_ref()
      .unwrap()
      .send(true)
      .expect("Could not send stop signal");

    if let Some(handle) = self.handle.take() {
      handle.join().unwrap();
    }
  }
}
