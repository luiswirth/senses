//! `others`: tells the agent of `AGENT_INBOX` the statuses of the other agents of its machine,
//! the inboxes beside its own, as a quiet item, as they change. Its inbox's take runs it at each
//! step; it does nothing without `AGENT_INBOX`.

use std::{env, path::PathBuf, process::ExitCode};

use agentinbox::Inbox;

fn main() -> ExitCode {
  if env::args().len() > 1 {
    eprintln!("usage: others, with AGENT_INBOX set; it takes no arguments");
    return ExitCode::FAILURE;
  }
  let Some(inbox) = env::var_os("AGENT_INBOX").map(PathBuf::from) else {
    return ExitCode::SUCCESS;
  };
  match others::tell(&Inbox::new(inbox)) {
    Ok(_) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("others: {error}");
      ExitCode::FAILURE
    }
  }
}
