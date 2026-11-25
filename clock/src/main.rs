//! `clock`: tells the agent of `MINDIO` the date and time, once 15 minutes
//! have passed since it last did. MindIO's take runs it, as a reading, at each step; it does nothing without
//! `MINDIO`.

use std::{env, path::PathBuf, process::ExitCode};

use jiff::{Timestamp, tz::TimeZone};
use mindio::MindIo;

fn main() -> ExitCode {
  if env::args().len() > 1 {
    eprintln!("usage: clock, with MINDIO set; it takes no arguments");
    return ExitCode::FAILURE;
  }
  let Some(mindio) = env::var_os("MINDIO").map(PathBuf::from) else {
    return ExitCode::SUCCESS;
  };
  match clock::tell(&MindIo::new(mindio), Timestamp::now(), &TimeZone::system()) {
    Ok(_) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("clock: {error}");
      ExitCode::FAILURE
    }
  }
}
