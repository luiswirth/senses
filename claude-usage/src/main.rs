//! `claude-usage`: tells the agent of `AGENT_INBOX` how much of its Claude subscription's usage
//! limits is used, as a quiet item, where a limit crossed a tenth since it was last told, either
//! way. A harness runs it at each step: it asks the endpoint at most once in `FRESH`, and does
//! nothing without `AGENT_INBOX` or a Claude Code login.

use std::{
  env,
  error::Error,
  fs::{self, TryLockError},
  io::{Read, Seek, SeekFrom, Write},
  os::unix::fs::OpenOptionsExt,
  path::PathBuf,
  process::ExitCode,
};

use claude_usage::{FRESH, SENDER, State, ask, create, deliver, tell, token};
use jiff::{Timestamp, tz::TimeZone};

fn main() -> ExitCode {
  if env::args().len() > 1 {
    eprintln!("usage: claude-usage, with AGENT_INBOX set; it takes no arguments");
    return ExitCode::FAILURE;
  }
  match run() {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("claude-usage: {error}");
      ExitCode::FAILURE
    }
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let Some(inbox) = env::var_os("AGENT_INBOX").map(PathBuf::from) else {
    return Ok(());
  };
  create(&inbox)?;
  let mut file = fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(false)
    .mode(0o600)
    .open(inbox.join(SENDER))?;
  // A run that holds the lock is asking already, such as for a parallel tool call.
  match file.try_lock() {
    Ok(()) => {}
    Err(TryLockError::WouldBlock) => return Ok(()),
    Err(TryLockError::Error(error)) => return Err(error.into()),
  }
  let mut saved = String::new();
  file.read_to_string(&mut saved)?;
  let mut state: State = serde_json::from_str(&saved).unwrap_or_default();

  let now = Timestamp::now();
  if state
    .asked
    .is_some_and(|asked| now.duration_since(asked).unsigned_abs() < FRESH)
  {
    return Ok(());
  }
  // Saved before asking, so a failing endpoint is asked no more often than a working one.
  state.asked = Some(now);
  save(&mut file, &state)?;

  let Some(token) = token()? else {
    return Ok(());
  };
  let usage = ask(&token)?;
  let text = tell(&mut state.told, &usage, &TimeZone::system());
  if !text.is_empty() {
    deliver(&inbox, &text)?;
  }
  save(&mut file, &state)
}

fn save(file: &mut fs::File, state: &State) -> Result<(), Box<dyn Error>> {
  file.set_len(0)?;
  file.seek(SeekFrom::Start(0))?;
  serde_json::to_writer(&mut *file, state)?;
  file.write_all(b"\n")?;
  Ok(())
}
