use std::collections::HashMap;
use std::str;

use crate::any::macros;

type IrcTagsMap<'a> = HashMap<&'a str, Option<&'a str>>;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct IrcInfo<'a> {
  pub(crate) user: &'a str,
  pub(crate) host: &'a str,
  pub(crate) text: &'a str,
  pub(crate) tags: IrcTagsMap<'a>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum TwitchIrcMsg<'a> {
  /// Sent when the bot or moderator removes all messages from the chat room or removes all messages for the specified user
  CLEARCHAT(IrcInfo<'a>),
  /// Sent when the bot removes a single message from the chat room
  CLEARMSG(IrcInfo<'a>),
  /// Sent when the bot authenticates with the server
  GLOBALUSERSTATE(IrcInfo<'a>),
  /// Sent to indicate the outcome of an action like banning a user
  NOTICE(IrcInfo<'a>),
  /// Sent when a user posts a message to the chat room
  PRIVMSG(IrcInfo<'a>),
  /// Sent when the bot joins a channel or when the channel’s chat room settings change
  ROOMSTATE(IrcInfo<'a>),
  /// Sent when events like someone subscribing to the channel occurs
  USERNOTICE(IrcInfo<'a>),
  /// Sent when the bot joins a channel or sends a PRIVMSG message
  USERSTATE(IrcInfo<'a>),
  /// Sent when someone sends your bot a whisper message
  WHISPER(IrcInfo<'a>),
  // Joined to a channel
  JOIN,
  // Ping: keep alive message
  PING,
  /// Nothing
  NOTHING,
}

fn parse_tags<'a>(tags: &'a str) -> IrcTagsMap<'a> {
  let mut map = HashMap::new();
  // Iterate over IRCv3 tags, separated by ';'
  // Example: "@badge-info=;badges=broadcaster/1,subscriber/0;color=#0000FF;..."
  for tag in tags.split(';') {
    let mut parts = tag.split('=');
    let key = parts.next().unwrap();
    let value = parts.next();
    map.insert(key, value);
  }
  map
}

pub(crate) fn parse_message<'a>(message: &'a str) -> TwitchIrcMsg<'a> {
  macros::log_inf!("message received {}", message);

  // Example: "@badge-info=;badges=broadcaster/1,... :tmi.twitch.tv PRIVMSG #channel :!test"
  // Parse tags if present
  let (tags, rest) = if message.starts_with('@') {
    let mut parts = message.splitn(2, ' ');
    let tags_part = parts.next().unwrap();
    (parse_tags(&tags_part[1..]), parts.next().unwrap_or(""))
  } else {
    (HashMap::new(), message)
  };

  // Split into prefix (user), command, and the remaining part
  // Example: ":tmi.twitch.tv PRIVMSG #channel :!test"
  let mut parts = rest.splitn(3, ' ');

  // Parse prefix part to get user
  let prefix_part = parts.next().unwrap_or("");
  let user = if prefix_part.starts_with(':') {
    // Example: ":tmi.twitch.tv!tmi.twitch.tv@tmi.twitch.tv"
    if let Some(user_host) = prefix_part.split('!').next() {
      &user_host[1..]
    } else {
      ""
    }
  } else {
    ""
  };
  // Parse command
  // Example: "PRIVMSG #channel :!test"
  let command = parts.next().unwrap_or("");

  // Parse remaining part of the message
  // Example: "#channel :!test"
  let remaining = parts.next().unwrap_or("");

  // Parse host and text from remaining part
  // Example: ("channel", "!test")
  let mut remaining = remaining.splitn(2, ' ');
  let host = remaining.next().unwrap_or("#").trim_start_matches('#');
  let text = remaining.next().unwrap_or(":").trim_start_matches(':');

  let irc_info = IrcInfo {
    user,
    host,
    text,
    tags,
  };

  // Match command to create specific IrcMsg variant
  match command {
    "CLEARCHAT" => TwitchIrcMsg::CLEARCHAT(irc_info),
    "CLEARMSG" => TwitchIrcMsg::CLEARMSG(irc_info),
    "GLOBALUSERSTATE" => TwitchIrcMsg::GLOBALUSERSTATE(irc_info),
    "NOTICE" => TwitchIrcMsg::NOTICE(irc_info),
    "PRIVMSG" => TwitchIrcMsg::PRIVMSG(irc_info),
    "ROOMSTATE" => TwitchIrcMsg::ROOMSTATE(irc_info),
    "USERNOTICE" => TwitchIrcMsg::USERNOTICE(irc_info),
    "USERSTATE" => TwitchIrcMsg::USERSTATE(irc_info),
    "WHISPER" => TwitchIrcMsg::WHISPER(irc_info),
    "JOIN" => TwitchIrcMsg::JOIN,
    "PING" => TwitchIrcMsg::PING,
    _ => TwitchIrcMsg::NOTHING,
  }
}
