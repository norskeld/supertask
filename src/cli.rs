use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
  #[command(subcommand)]
  pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
  /// Start supertask in the foreground.
  Start,
  /// Start supertask in the background.
  Daemon,
  /// Stop supertask and all running services.
  Stop,
  /// Show a summary of runtime data and config as a list.
  List {
    /// Show a table summary instead of a list.
    #[arg(short, long)]
    table: bool,
  },
  /// Give a single character summary of runtime state.
  Status,
  /// Clear specified events of errors and then run immediately.
  Clear {
    /// Events to clear.
    events: Vec<String>,
  },
  /// Run specified events immediately.
  Run {
    /// Events to run
    events: Vec<String>,
  },
  /// Delay events by specified intervals.
  Delay {
    /// Event to delay.
    event: String,
    /// Delay intervals.
    intervals: Vec<String>,
  },
  /// Show the config file schema.
  Schema,
  /// Show the config file path.
  Config,
  /// Show how interval expressions will be interpreted.
  Parse {
    /// Interval expressions to parse.
    expressions: Vec<String>,
  },
}
