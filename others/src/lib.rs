//! Tells an agent the statuses of the other agents of its machine, the inboxes beside its own,
//! as they change, so that it learns unasked who works where it does: each status set, changed
//! or cleared since it was last told. The first look tells them all, and how to set one's own.

use std::{
  collections::BTreeMap,
  fs::{self, TryLockError},
  io::{self, Read, Seek, SeekFrom, Write},
  os::unix::fs::OpenOptionsExt,
};

use agentinbox::{Inbox, inboxes};

/// Who the items are from, and the name of the sense's file in the inbox.
pub const SENDER: &str = "others";

/// The statuses of the agents beside this one, by name.
pub fn statuses(inbox: &Inbox) -> io::Result<BTreeMap<String, String>> {
  let mut statuses = BTreeMap::new();
  let Some(dir) = inbox.root.parent() else {
    return Ok(statuses);
  };
  let own = inbox.name();
  for other in inboxes(dir)? {
    let name = other.name();
    if name != own
      && let Some(status) = other.status()?
    {
      statuses.insert(name, status.text);
    }
  }
  Ok(statuses)
}

/// What changed from `told` to `now`, as the agent reads it, if anything did.
pub fn describe(
  told: Option<&BTreeMap<String, String>>,
  now: &BTreeMap<String, String>,
) -> Option<String> {
  let Some(told) = told else {
    let mut text = String::from("The other agents of this machine");
    if now.is_empty() {
      text.push_str(" have set no status.\n");
    } else {
      text.push_str(", by their statuses:\n");
      for (name, status) in now {
        text.push_str(&format!("- {name}: {status}\n"));
      }
    }
    text.push_str(
      "Their statuses are told here as they change. `agentinbox status <text>` sets yours, one \
       line on what you are doing and where, and `agentinbox agents` lists them all.\n",
    );
    return Some(text);
  };
  let mut lines = String::new();
  for (name, status) in now {
    if told.get(name) != Some(status) {
      lines.push_str(&format!("- {name}: {status}\n"));
    }
  }
  for name in told.keys().filter(|name| !now.contains_key(*name)) {
    lines.push_str(&format!("- {name}: no status any more\n"));
  }
  (!lines.is_empty()).then(|| format!("Other agents' statuses changed:\n{lines}"))
}

/// Delivers what changed in the others' statuses since they were last told, and says whether it
/// did. The file [`SENDER`] keeps what was told, one `<name>\t<status>` a line; a run that finds
/// it locked leaves the telling to the run that holds it, such as for a parallel tool call.
pub fn tell(inbox: &Inbox) -> io::Result<bool> {
  inbox.create()?;
  let path = inbox.root.join(SENDER);
  let first = !path.exists();
  let mut file = fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(false)
    .mode(0o600)
    .open(&path)?;
  match file.try_lock() {
    Ok(()) => {}
    Err(TryLockError::WouldBlock) => return Ok(false),
    Err(TryLockError::Error(error)) => return Err(error),
  }
  let mut text = String::new();
  file.read_to_string(&mut text)?;
  let told: BTreeMap<String, String> = text
    .lines()
    .filter_map(|line| line.split_once('\t'))
    .map(|(name, status)| (name.to_string(), status.to_string()))
    .collect();
  let now = statuses(inbox)?;
  let Some(news) = describe((!first).then_some(&told), &now) else {
    return Ok(false);
  };
  inbox.deliver(news.as_bytes(), SENDER, "txt")?;
  file.set_len(0)?;
  file.seek(SeekFrom::Start(0))?;
  for (name, status) in &now {
    writeln!(file, "{name}\t{status}")?;
  }
  Ok(true)
}
