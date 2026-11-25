//! Tells an agent how much of its Claude subscription's usage limits is used, through its inbox.

use std::{
  collections::BTreeMap,
  env, fs, io,
  path::{Path, PathBuf},
  process::{self, Command},
  time::Duration,
};

use jiff::{Timestamp, Unit, tz::TimeZone};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Who the items are from, and the name of the sense's file in the agent's IO.
pub const SENDER: &str = "claude-usage";

/// How long a reading stays fresh: the endpoint is asked at most once in this time.
pub const FRESH: Duration = Duration::from_secs(5 * 60);

/// The endpoint Claude Code reads its `/usage` from. It is undocumented.
const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";

/// What the sense keeps in the agent's IO between runs.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
  /// When the endpoint was last asked.
  pub asked: Option<Timestamp>,
  /// The tenth of each limit told last, by its key.
  pub told: BTreeMap<String, u8>,
}

/// The endpoint's answer, of which only the list of limits is read.
#[derive(Debug, Deserialize)]
pub struct Usage {
  #[serde(default)]
  pub limits: Vec<Limit>,
}

#[derive(Debug, Deserialize)]
pub struct Limit {
  /// The server's kind of limit, such as `session`, `weekly_all` or `weekly_scoped`.
  pub kind: String,
  pub percent: f64,
  pub resets_at: Option<Timestamp>,
  pub scope: Option<Scope>,
}

#[derive(Debug, Deserialize)]
pub struct Scope {
  pub model: Option<Model>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
  pub display_name: Option<String>,
}

impl Limit {
  fn model(&self) -> Option<&str> {
    self.scope.as_ref()?.model.as_ref()?.display_name.as_deref()
  }

  /// What tells this limit apart from the others.
  pub fn key(&self) -> String {
    match self.model() {
      Some(model) => format!("{}/{model}", self.kind),
      None => self.kind.clone(),
    }
  }

  pub fn name(&self) -> String {
    let kind = match self.kind.as_str() {
      "session" => "5-hour",
      "weekly_all" | "weekly_scoped" => "weekly",
      kind => kind,
    };
    match self.model() {
      Some(model) => format!("{kind} limit for {model}"),
      None => format!("{kind} limit"),
    }
  }

  /// The tenths of the limit used: the marks it is told by.
  pub fn tenths(&self) -> u8 {
    (self.percent / 10.0).floor().clamp(0.0, 10.0) as u8
  }

  pub fn describe(&self, zone: &TimeZone) -> String {
    let used = format!(
      "Your Claude subscription's {} is {}% used",
      self.name(),
      self.percent.round()
    );
    match self.resets_at {
      Some(time) => format!(
        "{used}, and resets {}.\n",
        // The server's times lie a fraction of a second around the full hour.
        time
          .round(Unit::Minute)
          .unwrap_or(time)
          .to_zoned_datetime(zone.clone())
          .strftime("%a %-d %b %H:%M %Z")
      ),
      None => format!("{used}.\n"),
    }
  }
}

/// What to tell of `usage`: each limit whose tenth changed since it was last told, either way,
/// a limit never told counting as at zero. Updates `told` to `usage`, forgetting the limits gone.
pub fn tell(told: &mut BTreeMap<String, u8>, usage: &Usage, zone: &TimeZone) -> String {
  let mut text = String::new();
  let mut now = BTreeMap::new();
  for limit in &usage.limits {
    let mark = limit.tenths();
    if told.get(&limit.key()).copied().unwrap_or(0) != mark {
      text += &limit.describe(zone);
    }
    now.insert(limit.key(), mark);
  }
  *told = now;
  text
}

/// The token of the user's Claude Code login, where Claude Code keeps it: on macOS in the
/// keychain, else in the file `.credentials.json` of its config directory. `None` where there is
/// no login. It is only read: Claude Code renews it as it runs.
pub fn token() -> io::Result<Option<String>> {
  let configured = env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from);
  let dir = match &configured {
    Some(dir) => dir.clone(),
    None => PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".claude"),
  };
  let credentials = match keychain(configured.as_deref())? {
    Some(credentials) => credentials,
    None => match fs::read_to_string(dir.join(".credentials.json")) {
      Ok(credentials) => credentials,
      Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
      Err(e) => return Err(e),
    },
  };

  #[derive(Deserialize)]
  #[serde(rename_all = "camelCase")]
  struct Credentials {
    claude_ai_oauth: Option<Oauth>,
  }
  #[derive(Deserialize)]
  #[serde(rename_all = "camelCase")]
  struct Oauth {
    access_token: String,
    expires_at: Option<i64>,
  }
  let credentials: Credentials = serde_json::from_str(&credentials)
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "unreadable Claude Code login"))?;
  let Some(oauth) = credentials.claude_ai_oauth else {
    return Ok(None);
  };
  if oauth
    .expires_at
    .is_some_and(|ms| ms < Timestamp::now().as_millisecond())
  {
    return Err(io::Error::other(
      "the Claude Code login has expired; Claude Code renews it as it runs",
    ));
  }
  Ok(Some(oauth.access_token))
}

/// The login in the macOS keychain, under the name Claude Code gives it: a config directory of
/// its own adds the start of its hash. `None` off macOS, or where it holds none.
fn keychain(configured: Option<&Path>) -> io::Result<Option<String>> {
  if !cfg!(target_os = "macos") {
    return Ok(None);
  }
  let mut service = "Claude Code-credentials".to_string();
  if let Some(dir) = configured {
    let hash: String = Sha256::digest(dir.as_os_str().as_encoded_bytes())
      .iter()
      .map(|byte| format!("{byte:02x}"))
      .collect();
    service = format!("{service}-{}", &hash[..8]);
  }
  let output = Command::new("security")
    .args(["find-generic-password", "-w", "-s", &service])
    .args(["-a", &env::var("USER").unwrap_or_default()])
    .stderr(process::Stdio::null())
    .output()?;
  Ok(
    output
      .status
      .success()
      .then(|| String::from_utf8_lossy(&output.stdout).into_owned()),
  )
}

/// Asks the endpoint for the usage of the login `token`.
pub fn ask(token: &str) -> Result<Usage, ureq::Error> {
  let agent: ureq::Agent = ureq::Agent::config_builder()
    .timeout_global(Some(Duration::from_secs(10)))
    .build()
    .into();
  agent
    .get(ENDPOINT)
    .header("Authorization", &format!("Bearer {token}"))
    .header("anthropic-beta", "oauth-2025-04-20")
    .header(
      "User-Agent",
      concat!("claude-usage/", env!("CARGO_PKG_VERSION")),
    )
    .call()?
    .body_mut()
    .read_json()
}
