use std::fs;

use agentinbox::{Content, Inbox};

fn told(inbox: &Inbox) -> Vec<String> {
  inbox
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
fn the_others_statuses_are_told_as_they_change() {
  let dir = std::env::temp_dir().join(format!("others-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&dir);
  // No peers of the user's.
  unsafe { std::env::set_var("XDG_CONFIG_HOME", dir.join("config")) };
  let me = Inbox::new(dir.join("me"));
  let lead = Inbox::new(dir.join("lead"));
  let fixer = Inbox::new(dir.join("fixer"));
  me.set_status("my own, never told").unwrap();
  lead.set_status("planning the week").unwrap();

  assert!(others::tell(&me).unwrap());
  let first = told(&me);
  assert_eq!(first.len(), 1);
  assert!(first[0].contains("- lead: planning the week\n"));
  assert!(!first[0].contains("my own"));

  assert!(!others::tell(&me).unwrap());

  fixer.set_status("fixing\nthe waker").unwrap();
  lead.set_status("").unwrap();
  assert!(others::tell(&me).unwrap());
  assert_eq!(
    told(&me),
    ["Other agents' statuses changed:\n- fixer: fixing the waker\n- lead: no status any more\n"]
  );
  assert!(!others::tell(&me).unwrap());
  fs::remove_dir_all(&dir).unwrap();
}
