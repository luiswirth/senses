use std::fs;

use agentinbox::{Content, Inbox};
use jiff::tz::TimeZone;

#[test]
fn the_time_is_told_once_15_minutes_have_passed() {
  let dir = std::env::temp_dir().join(format!("clock-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&dir);
  let inbox = Inbox::new(&dir);
  // A POSIX zone, since a build sandbox has no tz database.
  let zone = TimeZone::posix("CET-1CEST,M3.5.0,M10.5.0/3").unwrap();
  let at = |time: &str| clock::tell(&inbox, time.parse().unwrap(), &zone).unwrap();
  assert!(at("2026-11-18T17:05:00Z"));
  assert!(!at("2026-11-18T17:19:59Z"));
  assert!(at("2026-11-18T17:20:00Z"));
  let told: Vec<_> = inbox
    .take()
    .unwrap()
    .into_iter()
    .map(|item| match item.content {
      Content::Text(text) => text,
      Content::File { .. } => panic!("not text"),
    })
    .collect();
  assert_eq!(
    told,
    [
      "Wednesday 2026-11-18 18:05 +01:00\n",
      "Wednesday 2026-11-18 18:20 +01:00\n"
    ]
  );
  fs::remove_dir_all(&dir).unwrap();
}
