use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::macros;

#[derive(Deserialize)]
pub(crate) struct AuthTokens {
  pub(crate) spotify: Option<String>,
  pub(crate) twitch: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Reply {
  pub(crate) name: Option<String>,
  pub(crate) reply: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Pattern {
  pub(crate) regex: Option<String>,
  pub(crate) reply: Option<String>,
  pub(crate) chance: Option<u32>,
}

#[derive(Deserialize)]
pub(crate) struct Runner {
  pub(crate) regex: Option<String>,
  pub(crate) run: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Interactions {
  pub(crate) replies: Option<Vec<Reply>>,
  pub(crate) patterns: Option<Vec<Pattern>>,
  pub(crate) runners: Option<Vec<Runner>>,
}

#[derive(Deserialize)]
pub(crate) struct ConfigMe {
  pub(crate) channel: Option<String>,
  pub(crate) name: Option<String>,
  pub(crate) auth: AuthTokens,
  pub(crate) interact: Interactions,
}

macro_rules! select_value {
  ($a:expr, $b:expr) => {
    match ($a, $b) {
      (Some(val), _) => Some(val),
      (_, Some(fal)) => fal,
      (None, None) => None,
    }
  };
}

macro_rules! ensure_value {
  ($a:expr) => {
    match $a {
      None => {
        macros::exit_err!("");
      }
      _ => $a,
    }
  };
}

macro_rules! check {
  ($e:expr) => {
    match $e {
      Ok(t) => t,
      Err(e) => panic!("{} failed with: {e}", stringify!($e)),
    }
  };
}

pub(crate) fn load(path: &Path) -> Option<ConfigMe> {
  let r = check!(std::fs::read_to_string(path.to_owned()));
  let cfg: ConfigMe = check!(serde_json::from_str(r.as_str()));

  Some(ConfigMe {
    channel: ensure_value!(cfg.channel),
    name: ensure_value!(cfg.name),
    auth: AuthTokens {
      spotify: ensure_value!(cfg.auth.spotify),
      twitch: ensure_value!(cfg.auth.twitch),
    },
    interact: Interactions {
      replies: select_value!(cfg.interact.replies, None),
      patterns: select_value!(cfg.interact.patterns, None),
      runners: select_value!(cfg.interact.runners, None),
    },
  })
}
