mod any;
mod bot;
mod cli;

fn main() {
  let exec_name: String = std::env::args()
    .nth(0)
    .unwrap()
    .split(std::path::MAIN_SEPARATOR)
    .collect::<Vec<&str>>()[..]
    .last()
    .unwrap()
    .to_string();

  match exec_name.as_str() {
    "lyrbot" => println!("Sorry, feature not implemented yet"),
    "lyrgui" => println!("Sorry, feature not implemented yet"),
    _ => cli::cli(),
  }
}
