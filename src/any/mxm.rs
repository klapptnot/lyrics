use fancy_regex::Regex;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::time::Duration;

use crate::any::{macros, spinner::Spinner};

pub(crate) struct TrackItem {
  pub(crate) url: String,
  pub(crate) desc: String,
}

impl TrackItem {
  pub(crate) fn new(url: String, desc: String) -> Self {
    Self { url, desc }
  }
}

pub(crate) struct LyricParagraph {
  pub(crate) lines: Vec<String>,
  pub(crate) title: String, // Discard type, its always lyrics
}

impl LyricParagraph {
  fn vec_of(data: &Value) -> Vec<LyricParagraph> {
    let mut me: Vec<LyricParagraph> = Vec::new();
    if let Value::Array(data) = data {
      for lp in data {
        me.push(LyricParagraph {
          lines: lp["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| Value::as_str(&l["text"]).unwrap().into())
            .collect::<Vec<String>>(),
          title: Value::as_str(&lp["title"]).unwrap().into(),
        });
      }
    }
    me
  }
}

/// Trying to not have this in main.rs, here is it
#[allow(dead_code)]
pub(crate) struct TrackInfo {
  /// Song title
  pub(crate) name: String,
  /// Song artist/group
  pub(crate) artist: String,
  /// Album containing the song
  pub(crate) album: String,
  /// Whether song has lyrics or lyrics is disponible
  pub(crate) has_lyrics: bool,
  /// Whether song has lyrics structure or not
  pub(crate) has_lyrics_struct: bool,
  /// Lyrics (full) language name
  pub(crate) lyrics_lang: String,
  /// The lyrics as string (escaped)
  pub(crate) lyrics: String,
  /// A structured lyrics representation
  pub(crate) lyrics_struct: Vec<LyricParagraph>,
  /// Lyric composer(s)
  pub(crate) lyrics_copyright: String,
  /// Song primary genre
  pub(crate) genre: String,
  /// Cover image URL (expect a 350x350 px jpg file url)
  pub(crate) cover: String,
  /// Song release date in format YYYY-MM-DD
  pub(crate) released: String,
  /// Spotify URL for this song
  pub(crate) spotify: String,
  /// Musixmatch URL for this song
  pub(crate) musixmatch: String,
}

impl TrackInfo {
  fn from(json: String) -> Option<Self> {
    let data: Option<Value> = serde_json::from_str(json.as_str()).unwrap_or(None);
    if let Some(data) = data {
      return Some(Self {
        name: Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["name"])
          .unwrap_or("Unespecified")
          .into(),
        artist: Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["artistName"])
          .unwrap_or("Unespecified")
          .into(),
        album: Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["albumName"])
          .unwrap_or("Unespecified")
          .into(),
        has_lyrics: Value::as_bool(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["hasLyrics"])
          .unwrap_or(false),
        has_lyrics_struct: Value::as_bool(
          &data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["hasTrackStructure"],
        )
        .unwrap_or(false),
        lyrics_lang: Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["lyrics"]["languageDescription"])
          .unwrap_or("Unespecified")
          .into(),
        lyrics: Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["lyrics"]["body"])
          .unwrap_or("Unespecified")
          .into(),
        lyrics_struct: LyricParagraph::vec_of(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["trackStructureList"]),
        lyrics_copyright: Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["lyrics"]["copyright"])
          .unwrap_or("Unespecified")
          .into(),
        genre: Value::as_str(
          &data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["artists"][0]["genres"][0]["name"],
        )
        .unwrap_or("Unespecified")
        .into(),
        cover: Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["coverImage"])
          .unwrap_or("Unespecified")
          .into(),
        released: Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["releaseDate"])
          .unwrap_or("0000-00-00")[0..10]
          .into(),
        spotify: if let Some(s) = Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["spotifyId"])
        {
          format!("https://open.spotify.com/track/{s}")
        } else {
          "Missing Spotify music ID".to_string()
        },
        musixmatch: if let Some(s) =
          Value::as_str(&data["props"]["pageProps"]["data"]["trackInfo"]["data"]["track"]["vanityId"])
        {
          format!("https://musixmatch.com/lyrics/{s}")
        } else {
          "Missing Musixmatch ID".to_string()
        },
      });
    }
    return None;
  }
}

pub struct MxmAPI {
  tries: u32,
  timeout: u32,
  headers: Option<HeaderMap>,
}

pub enum ResponseErr {
  Captcha,
  JsEnforcement,
  NoEnoughData,
  RequestErr,
}

impl MxmAPI {
  pub fn new(tries: u32, timeout: u32, headers: Option<HeaderMap>) -> Self {
    Self {
      tries,
      timeout,
      headers,
    }
  }

  pub fn get_from_url(&self, url: &String) -> TrackInfo {
    let mut mxm_json: Option<String> = None;
    let mut spinner = Spinner::new();
    spinner.start("Getting song data".into());

    for i in 1..=self.tries {
      if let Ok(json_str) = get_json(&url, self.timeout, self.headers.clone()) {
        mxm_json = Some(json_str);
        break;
      } else {
        spinner.update(format!("Getting song data ({} try)", i + 1));
      }
    }

    spinner.stop();
    let mxm_json: String = mxm_json.unwrap_or_else(|| {
      macros::exit_err!("Couldn't get a (valid) response from the server");
    });
    let track = TrackInfo::from(mxm_json).unwrap_or_else(|| {
      macros::exit_err!("Couldn't read json from response");
    });

    track
  }

  pub fn get_track_info(&self, keyword: &String, index: usize) -> TrackInfo {
    let uti = self.get_possible_links(keyword);
    let uti = uti.get(index).unwrap();

    MxmAPI::get_from_url(&self, &uti.url)
  }

  pub fn get_possible_links(&self, keyword: &String) -> Vec<TrackItem> {
    let mut spinner = Spinner::new();
    spinner.start("Getting url to musixmatch".into());

    let mut urls_list: Option<Vec<TrackItem>> = None;
    for i in 1..=self.tries {
      match get_urls(&keyword, self.timeout, self.headers.clone()) {
        Ok(val) => {
          urls_list = Some(val);
          break;
        }
        Err(ResponseErr::Captcha) => {
          macros::exit_err!("Captcha triggered, page has no data");
        }
        Err(ResponseErr::JsEnforcement) => {
          macros::exit_err!("JavaScript enforcement, page has no data");
        }
        _ => {
          spinner.update(format!("Getting url to musixmatch ({} try)", i + 1));
        }
      }
    }

    spinner.stop();
    let uti = urls_list.unwrap_or_else(|| {
      macros::exit_err!("There are no results for this query");
    });

    uti
  }
}

pub fn get_urls(keyword: &String, timeout: u32, headers_map: Option<HeaderMap>) -> Result<Vec<TrackItem>, ResponseErr> {
  let url = reqwest::Url::parse(
    format!(
      "https://www.google.com/search?q=site%3Amusixmatch.com%2Flyrics%20lyrics%20{}",
      keyword
    )
    .as_str(),
  )
  .unwrap();

  let client = reqwest::blocking::Client::new()
    .get(url)
    .timeout(Duration::from_millis(timeout as u64))
    .headers(headers_map.unwrap_or(HeaderMap::new()));

  let res = client.send();
  let response = match res {
    Ok(data) => data.text().unwrap(),
    Err(_) => return Err(ResponseErr::RequestErr),
  };

  if response.contains(r#"/httpservice/retry/enablejs"#) {
    return Err(ResponseErr::JsEnforcement);
  }

  if response.contains(r#"<script src="https://www.google.com/recaptcha/api.js" async defer></script>"#) {
    return Err(ResponseErr::Captcha);
  }

  let url_list: Vec<String> = Regex::new(r#"(?<=><a jsname="UWckNb" href=")[^ ]*(?=")"#)
    .unwrap()
    .find_iter(response.as_str())
    .map(|m| String::from(m.unwrap().as_str()))
    .collect();

  if url_list.len() == 0 {
    return Err(ResponseErr::NoEnoughData);
  }

  let url_desc: Vec<String> = Regex::new(r#"(?<=<br><h3 class="LC20lb MBeuO DKV0Md">)[^<]*(?=<)"#)
    .unwrap()
    .find_iter(response.as_str())
    .map(|m| String::from(m.unwrap().as_str()))
    .collect();

  let mut urls_tp: Vec<TrackItem> = Vec::new();
  let tr_re = Regex::new(r"/translation/.*$").unwrap();

  // Add all not translation URLs
  for i in 0..url_list.len() {
    if tr_re.is_match(url_list[i].as_str()).unwrap() {
      let uns_url = String::from(tr_re.replace(url_list[i].as_str(), ""));
      if url_list.contains(&uns_url) {
        continue;
      }
      urls_tp.push(TrackItem::new(uns_url, String::from(&url_desc[i])))
    } else {
      urls_tp.push(TrackItem::new(
        String::from(&url_list[i]),
        String::from(&url_desc[i]),
      ))
    }
  }

  Ok(urls_tp)
}

fn get_json(url: &String, timeout: u32, headers_map: Option<HeaderMap>) -> Result<String, String> {
  let r_url = Regex::new(r#"^https?://(?:www\.)?musixmatch\.com/"#).unwrap();
  if !r_url.is_match(&url.as_str()).unwrap_or(false) {
    // URL must be valid (i.e. not empty, from musixmatch)
    return Err("Invalid URL".into());
  }

  let client = reqwest::blocking::Client::new()
    .get(url)
    .timeout(Duration::from_millis(timeout as u64))
    .headers(headers_map.unwrap_or(HeaderMap::new()));

  let response: reqwest::blocking::Response;
  let res = client.send();
  response = match res {
    Err(_) => return Err("Something went wrong".into()),
    Ok(data) => data,
  };

  let response = response.text().unwrap_or_else(|e| {
    macros::exit_err!("Could not get response content: {e}");
  });
  let json_res = Regex::new(r#"(?<=<script id="__NEXT_DATA__" type=\"application/json">).*(?=</script>)"#)
    .unwrap()
    .find(response.as_str())
    .unwrap();

  if json_res.is_none() {
    return Err("Invalid response structure".into());
  }

  return Ok(json_res.unwrap().as_str().into());
}
