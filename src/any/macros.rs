#[allow(unused_macros)]
macro_rules! exit_err {
  ($($arg:expr),*) => {
    eprintln!("\x1b[38;5;197m[ FATAL ] -> {}\x1b[0m", format!($($arg),*));
    std::process::exit(1)
  };
}

#[allow(unused_macros)]
macro_rules! log_err {
  ($($arg:expr),*) => {
    eprintln!("\x1b[38;5;209m[ ERROR ] -> {}\x1b[0m", format!($($arg),*));
  };
}

#[allow(unused_macros)]
macro_rules! log_inf {
  ($($arg:expr),*) => {
    eprintln!("\x1b[38;5;51m[ INFO ] -> {}\x1b[0m", format!($($arg),*));
  };
}

#[allow(unused_macros)]
macro_rules! log_ok {
  ($($arg:expr),*) => {
    eprintln!("\x1b[38;5;48m[ OKAY ] -> {}\x1b[0m", format!($($arg),*));
  };
}

#[allow(unused_imports)]
pub(crate) use exit_err;
#[allow(unused_imports)]
pub(crate) use log_err;
#[allow(unused_imports)]
pub(crate) use log_inf;
#[allow(unused_imports)]
pub(crate) use log_ok;
