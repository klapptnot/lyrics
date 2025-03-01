use crate::any::{
  macros,
  mxm::{MxmAPI, TrackInfo},
  uagent,
};
use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue, HeaderName};
use std::io::{Read, Write};

#[derive(Parser)]
struct Args {
  ///The search query to find the music
  query: Vec<String>,
  ///Url to use instead of searching one
  #[clap(short = 'u', long = "url", default_value = None)]
  url: Option<String>,
  ///Timeout timeout in milliseconds
  #[clap(short = 't', long = "timeout", default_value = "5000")]
  timeout: u32,
  ///Number of trying attempts to get data
  #[clap(short = 'T', long = "tries", default_value = "5")]
  tries: u32,
  ///URL index, use -a to view all URLs
  #[clap(short = 'i', long = "url-index", default_value = "0")]
  url_index: usize,
  // ---------------------------------------
  // Changes to MxmAPI needed
  //
  // ///Proxy address to use
  // #[clap(short = 'p', long = "proxy", default_value = None)]
  // proxy: Option<String>,
  // ///Proxy list file to read from
  // #[clap(short = 'P', long = "proxylist", default_value = None)]
  // proxylist: Option<String>,
  // ///Cookie string for musixmatch to bypass captcha
  // #[clap(short = 'm', long = "mxm-cookies", default_value = None)]
  // mxm_cookie: Option<String>,
  // ---------------------------------------
  ///User agent string
  #[clap(short = 'U', long = "user-agent", default_value = None)]
  user_agent: Option<String>,
  ///Show URL found and ask user to select one
  #[clap(short = 'a', long = "tip-url", default_value = "false")]
  typ_url: bool,
  ///Only print lyrics (With -r is a bit different)
  #[clap(short = 'l', long = "lyrics", default_value = "false")]
  only_lyrics: bool,
  ///Show album cover art (prefer using a terminal with image support)
  #[clap(short = 'c', long = "cover-art", default_value = "false")]
  show_cover: bool,
  ///Print <Track name> - <Artist> before printing lyrics
  #[clap(short = 'r', long = "repeat", default_value = "false")]
  repeat: bool,
}

/// The CLI functionality
pub fn cli() {
  let args = Args::parse();

  // Parse some command line arguments items as groups
  if args.query.len() < 1 && args.url.is_none() {
    macros::exit_err!("You must specify a url or query to get a url");
  } else if !args.query.len() < 1 && !args.url.is_none() {
    macros::exit_err!("Cannot specify a url and query at the same time");
  }

  if args.tries == 0 {
    macros::exit_err!("--tries/-T cannot accept 0");
  }

  let mut headers = HeaderMap::new();
  if args.user_agent.is_none() {
    headers.insert(reqwest::header::USER_AGENT, uagent::random().parse().unwrap());
  } else {
    headers.insert(
      reqwest::header::USER_AGENT,
      args.user_agent.unwrap().parse().unwrap(),
    );
  }

  headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
  headers.insert(reqwest::header::ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
  headers.insert(reqwest::header::ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
  headers.insert(reqwest::header::DNT, HeaderValue::from_static("1"));
  headers.insert(reqwest::header::CONNECTION, HeaderValue::from_static("disconnect"));
  headers.insert(reqwest::header::UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));
  headers.insert(HeaderName::from_static("sec-gpc"), HeaderValue::from_static("1").into());
  headers.insert(HeaderName::from_static("sec-fetch-dest"), HeaderValue::from_static("document").into());
  headers.insert(HeaderName::from_static("sec-fetch-mode"), HeaderValue::from_static("navigate").into());
  headers.insert(HeaderName::from_static("sec-fetch-site"), HeaderValue::from_static("none").into());
  headers.insert(HeaderName::from_static("sec-fetch-user"), HeaderValue::from_static("?1").into());
  headers.insert(HeaderName::from_static("priority"), HeaderValue::from_static("u=0, i").into());

  let mxm_api = MxmAPI::new(args.tries, args.timeout, Some(headers));
  let track: TrackInfo;

  if !args.url.is_none() {
    track = mxm_api.get_from_url(&args.url.unwrap());
  } else {
    let kwds = args
      .query
      .iter()
      .map(|e| e.as_str())
      .collect::<Vec<&str>>()
      .join(" ");

    if !args.typ_url {
      track = mxm_api.get_track_info(&kwds, args.url_index);
    } else {
      let urls = mxm_api.get_possible_links(&kwds);
      println!("\x1b[38;2;195;79;230mAvailable options are:\x1b[0m");
      for i in 0..urls.len() {
        println!(
        "  {} \x1b[38;2;255;169;140m-> \x1b[38;2;255;232;184m{}\n    \x1b[38;2;195;79;230mAt: \x1b[38;2;189;147;249m{}\x1b[0m",
        i, urls[i].desc, urls[i].url
      );
      }
      print!("\x1b[38;2;195;79;230mSelect one from above:\x1b[0m ");
      std::io::stdout().flush().unwrap();
      let mut idx: [u8; 1] = [48];
      std::io::stdin()
        .read_exact(&mut idx)
        .expect("Could not read the input");

      // Ascii(48-57) == Ordinal(0-9)
      if idx[0] > 57 || idx[0] < 48 {
        macros::exit_err!("Invalid input, select a number");
      }

      let idx: usize = (idx[0] - 48) as usize;

      track = if let Some(u) = &urls.get(idx) {
        mxm_api.get_from_url(&u.url)
      } else {
        macros::exit_err!("Index {idx} is out of bounds");
      };
    }
  }

  // let track = TrackInfo::from(crate::dummy::get_json()).unwrap_or_else(|| macros::exit_err("Not able to get TrackInfo"));

  if args.only_lyrics {
    if !track.has_lyrics {
      macros::exit_err!("This song has no lyrics or lyrics are not available");
    }
    if args.repeat {
      println!(
        "\x1b[38;2;195;79;230m{}\x1b[38;2;223;225;255m - \x1b[38;2;189;147;249m{}\x1b[0m\n",
        track.name, track.artist
      );
    }
    // This has no custom color to use it in pipes properly
    println!("{}", track.lyrics);
    std::process::exit(0);
  }

  let keyc = "\x1b[38;2;255;169;140m";
  let valc = "\x1b[38;2;255;232;184m";
  let img_pad = if args.show_cover { 16 } else { 0 };

  // Defaults to all relevant info
  println!("{:img_pad$}{keyc}TITLE     : {valc}{}", "", track.name);
  println!("{:img_pad$}{keyc}ARTIST    : {valc}{}", "", track.artist);
  println!("{:img_pad$}{keyc}ALBUM     : {valc}{}", "", track.album);
  println!("{:img_pad$}{keyc}GENRE     : {valc}{}", "", track.genre);
  println!("{:img_pad$}{keyc}RELEASED  : {valc}{}", "", track.released);
  println!("{:img_pad$}{keyc}SPOTIFY   : {valc}{}", "", track.spotify);
  println!("{:img_pad$}{keyc}MUSIXMATCH: {valc}{}", "", track.musixmatch);

  if args.show_cover {
    let conf = viuer::Config {
      x: 0,
      y: -7,
      restore_cursor: true,
      absolute_offset: false,
      width: Some(15),
      height: Some(7),
      ..Default::default()
    };

    let response = reqwest::blocking::get(track.cover).expect("get album cover art");
    let bytes = response.bytes().expect("get response bytes");
    let img = image::load_from_memory(&bytes).expect("decode image");

    viuer::print(&img, &conf).expect("print image");
  }

  if !track.has_lyrics {
    print!("\nLyrics are not available :(");
    std::process::exit(0);
  }

  print!("\n{keyc}LYRICS\x1b[0m\n\n");

  if !track.has_lyrics_struct {
    print!("{}\n\nCopyright -> {}\n", track.lyrics, track.lyrics_copyright);
    std::process::exit(0)
  }
  for paragraph in track.lyrics_struct {
    println!("\x1b[38;2;189;147;249m#[section({})]\x1b[0m", paragraph.title);
    for line in paragraph.lines {
      print!("{}\n", line);
    }
    print!("\n\n")
  }
  // For now, it has a trailing '\n'
  print!("Copyright -> {}", track.lyrics_copyright);
}
