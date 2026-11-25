//! `others`: tells the agent of `MINDIO` the statuses of the other agents of its machine,
//! the agents beside it, as they change. MindIO's take runs it, as a reading, at each
//! step; it does nothing without `MINDIO`.

use std::{env, path::PathBuf, process::ExitCode};

use mindio::MindIo;

fn main() -> ExitCode {
  if env::args().len() > 1 {
    eprintln!("usage: others, with MINDIO set; it takes no arguments");
    return ExitCode::FAILURE;
  }
  let Some(mindio) = env::var_os("MINDIO").map(PathBuf::from) else {
    return ExitCode::SUCCESS;
  };
  match others::tell(&MindIo::new(mindio)) {
    Ok(_) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("others: {error}");
      ExitCode::FAILURE
    }
  }
}
