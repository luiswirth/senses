use std::fs;

use mindio::{Content, MindIo};

fn told(mindio: &MindIo) -> Vec<String> {
  mindio
    .inbox()
    .take()
    .unwrap()
    .into_iter()
    .map(|item| match item.content {
      Content::Text(text) => text,
      Content::File { .. } => panic!("not text"),
    })
    .collect()
}

#[test]
fn the_other_agents_statuses_are_told_as_they_change() {
  let dir = std::env::temp_dir().join(format!("agent-statuses-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&dir);
  // No peers of the user's.
  unsafe { std::env::set_var("XDG_CONFIG_HOME", dir.join("config")) };
  let me = MindIo::new(dir.join("me"));
  let lead = MindIo::new(dir.join("lead"));
  let fixer = MindIo::new(dir.join("fixer"));
  me.set_status("my own, never told").unwrap();
  lead.set_status("planning the week").unwrap();

  assert!(agent_statuses::tell(&me).unwrap());
  let first = told(&me);
  assert_eq!(first.len(), 1);
  assert!(first[0].contains("- lead: planning the week\n"));
  assert!(!first[0].contains("my own"));

  assert!(!agent_statuses::tell(&me).unwrap());

  fixer.set_status("fixing\nthe waker").unwrap();
  lead.set_status("").unwrap();
  assert!(agent_statuses::tell(&me).unwrap());
  assert_eq!(
    told(&me),
    ["Other agents' statuses changed:\n- fixer: fixing the waker\n- lead: no status any more\n"]
  );
  assert!(!agent_statuses::tell(&me).unwrap());
  fs::remove_dir_all(&dir).unwrap();
}
