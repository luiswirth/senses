use std::{collections::BTreeMap, fs};

use claude_usage::{Usage, create, deliver, tell};
use jiff::tz::TimeZone;

/// The endpoint's answer, as it came on 2026-11-16, with the fields the sense ignores left out.
fn usage(session: u32, weekly: u32) -> Usage {
  serde_json::from_str(&format!(
    r#"{{"five_hour": {{"utilization": {session}.0}}, "limits": [
      {{"kind": "session", "group": "session", "percent": {session}, "resets_at": "2026-11-16T17:59:59.931711+00:00", "scope": null}},
      {{"kind": "weekly_all", "group": "weekly", "percent": {weekly}, "resets_at": "2026-11-21T20:00:00.131728+00:00", "scope": null}},
      {{"kind": "weekly_scoped", "group": "weekly", "percent": 2, "resets_at": "2026-11-21T20:00:00.131872+00:00", "scope": {{"model": {{"id": null, "display_name": "Fable"}}, "surface": null}}}}
    ]}}"#
  ))
  .unwrap()
}

#[test]
fn a_limit_is_told_as_it_crosses_a_tenth_either_way() {
  let zone = TimeZone::get("Europe/Zurich").unwrap();
  let mut told = BTreeMap::new();
  assert_eq!(
    tell(&mut told, &usage(11, 26), &zone),
    "Your Claude subscription's 5-hour limit is 11% used, and resets Mon 16 Nov 19:00 CET.\n\
     Your Claude subscription's weekly limit is 26% used, and resets Sat 21 Nov 21:00 CET.\n"
  );
  assert_eq!(tell(&mut told, &usage(19, 29), &zone), "");
  assert_eq!(
    tell(&mut told, &usage(20, 29), &zone),
    "Your Claude subscription's 5-hour limit is 20% used, and resets Mon 16 Nov 19:00 CET.\n"
  );
  assert_eq!(
    tell(&mut told, &usage(0, 29), &zone),
    "Your Claude subscription's 5-hour limit is 0% used, and resets Mon 16 Nov 19:00 CET.\n"
  );
}

#[test]
fn a_limit_of_a_model_is_named_by_it() {
  let zone = TimeZone::UTC;
  let mut told = BTreeMap::new();
  let mut usage = usage(0, 0);
  usage.limits[2].percent = 55.0;
  assert_eq!(
    tell(&mut told, &usage, &zone),
    "Your Claude subscription's weekly limit for Fable is 55% used, and resets Sat 21 Nov 20:00 UTC.\n"
  );
}

#[test]
fn an_item_is_delivered_whole_into_new() {
  let inbox = std::env::temp_dir().join(format!("claude-usage-{}", std::process::id()));
  create(&inbox).unwrap();
  let item = deliver(&inbox, "text\n").unwrap();
  assert_eq!(item.parent().unwrap(), inbox.join("new"));
  assert!(item.to_str().unwrap().ends_with(".claude-usage.txt"));
  assert_eq!(fs::read_to_string(&item).unwrap(), "text\n");
  assert_eq!(fs::read_dir(inbox.join("tmp")).unwrap().count(), 0);
  fs::remove_dir_all(&inbox).unwrap();
}
