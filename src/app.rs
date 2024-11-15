use std::process;

use clap::Parser;

use crate::cli::{Cli, Commands};

#[derive(Debug)]
enum Status {
  Running,
  Stopped,
}

impl Status {
  fn current() -> Self {
    Self::Running
  }
}

pub fn run() {
  let cli = Cli::parse();

  ensure_default_directory();

  match cli.command {
    | Commands::Start => {
      match Status::current() {
        | Status::Running => bail("running"),
        | Status::Stopped => {
          save_pid();
          run_bot();
        },
      }
    },
    | Commands::Daemon => {
      match Status::current() {
        | Status::Running => bail("running"),
        | Status::Stopped => run_daemon(),
      }
    },
    | Commands::Stop => {
      match Status::current() {
        | Status::Running => stop_daemon(),
        | Status::Stopped => bail("stopped"),
      }
    },
    | Commands::List { table } => run_list(table),
    | Commands::Status => run_status(),
    | Commands::Clear { events } => events.into_iter().for_each(clear),
    | Commands::Run { events } => events.into_iter().for_each(execute),
    | Commands::Delay { event, intervals } => delay(event, intervals),
    | Commands::Schema => run_schema(),
    | Commands::Config => {
      match get_config_path() {
        | Ok(path) => println!("{}", path),
        | Err(e) => eprintln!("Error getting config path: {}", e),
      }
    },
    | Commands::Parse { expressions } => run_parse_check(expressions),
  }
}

fn bail(status: &str) {
  eprintln!("devbot appears to already be {}", status);
  process::exit(1);
}

fn ensure_default_directory() {
  todo!()
}

fn save_pid() {
  todo!()
}

fn run_bot() {
  todo!()
}

fn run_daemon() {
  todo!()
}

fn stop_daemon() {
  todo!()
}

fn run_list(_table: bool) {
  todo!()
}

fn run_status() {
  todo!()
}

fn clear(_event: String) {
  todo!()
}

fn execute(_event: String) {
  todo!()
}

fn delay(_event: String, _intervals: Vec<String>) {
  todo!()
}

fn run_schema() {
  todo!()
}

fn get_config_path() -> Result<String, std::io::Error> {
  Ok("/path/to/config".to_string())
}

fn run_parse_check(_expressions: Vec<String>) {
  todo!()
}
