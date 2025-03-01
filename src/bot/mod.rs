mod irc;
mod twitch;
// mod config;
// mod spotify;

use crate::any::macros;
use std::{io::BufRead, rc::Rc};

pub(crate) fn bot(channel: &str, username: &str, token: &str) {
  let mut twitch = twitch::TwitchHandler::new(channel, username);
  let twitch = Rc::from(twitch.login(token));

  let reader = Rc::clone(&twitch).get_reader();
  let mut lines = reader.lines();

  while let Some(Ok(ln)) = lines.next() {
    let msg = irc::parse_message(&ln.as_str());
    macros::log_inf!("Message: {:#?}", msg);

    match msg {
      irc::TwitchIrcMsg::JOIN => {
        macros::log_ok!("Successfully authenticated and joined {}", channel);
      }
      // irc::TwitchIrcMsg::PING => {
      //   twitch::socket_send_raw(&mut Rc::clone(&twitch).socket, "PONG :tmi.twitch.tv".to_string());
      // },
      _ => (),
    };
  }
}
