//! Tells an agent the date and time, through its inbox, once [`GAP`] has passed since it last
//! did: the clock is outside the agent, so a reading of its inbox, while how long a step took is
//! the harness's own.

use std::{
  fs::{self, TryLockError},
  io::{self, Read, Seek, SeekFrom, Write},
  os::unix::fs::OpenOptionsExt,
};

use agentinbox::Inbox;
use jiff::{SignedDuration, Timestamp, tz::TimeZone};

/// Who the items are from, and the name of the sense's file in the inbox.
pub const SENDER: &str = "clock";

/// How much time passes before the date and time are told again.
pub const GAP: SignedDuration = SignedDuration::from_mins(15);

/// The date and time as the agent reads them, in `zone`.
pub fn describe(now: Timestamp, zone: &TimeZone) -> String {
  let name = zone
    .iana_name()
    .map(|name| format!(" ({name})"))
    .unwrap_or_default();
  format!(
    "{}{name}\n",
    now
      .to_zoned_datetime(zone.clone())
      .strftime("%A %Y-%m-%d %H:%M %:z")
  )
}

/// Delivers the date and time where [`GAP`] has passed since they were last told, and says
/// whether it did. The file [`SENDER`] keeps when; a run that finds it locked leaves the telling
/// to the run that holds it, such as for a parallel tool call.
pub fn tell(inbox: &Inbox, now: Timestamp, zone: &TimeZone) -> io::Result<bool> {
  inbox.create()?;
  let mut file = fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(false)
    .mode(0o600)
    .open(inbox.root.join(SENDER))?;
  match file.try_lock() {
    Ok(()) => {}
    Err(TryLockError::WouldBlock) => return Ok(false),
    Err(TryLockError::Error(error)) => return Err(error),
  }
  let mut told = String::new();
  file.read_to_string(&mut told)?;
  if let Ok(told) = told.trim().parse::<Timestamp>()
    && now.duration_since(told) < GAP
  {
    return Ok(false);
  }
  inbox.deliver(describe(now, zone).as_bytes(), SENDER, "txt")?;
  file.set_len(0)?;
  file.seek(SeekFrom::Start(0))?;
  writeln!(file, "{now}")?;
  Ok(true)
}
