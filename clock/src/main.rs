//! `clock`: tells the agent of `AGENT_INBOX` the date and time, as a quiet item, once 15 minutes
//! have passed since it last did. Its inbox's take runs it at each step; it does nothing without
//! `AGENT_INBOX`.

use std::{env, path::PathBuf, process::ExitCode};

use agentinbox::Inbox;
use jiff::{Timestamp, tz::TimeZone};

fn main() -> ExitCode {
  if env::args().len() > 1 {
    eprintln!("usage: clock, with AGENT_INBOX set; it takes no arguments");
    return ExitCode::FAILURE;
  }
  let Some(inbox) = env::var_os("AGENT_INBOX").map(PathBuf::from) else {
    return ExitCode::SUCCESS;
  };
  match clock::tell(&Inbox::new(inbox), Timestamp::now(), &TimeZone::system()) {
    Ok(_) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("clock: {error}");
      ExitCode::FAILURE
    }
  }
}
