use grlm::AuthType;

use docopt::Docopt;
use libc;
use std::convert::From;
use std::fmt;

#[derive(Debug, Deserialize)]
struct Arguments {
    flag_login: String,
    flag_password: String,
    flag_access_token: String,
    flag_frequency: u64,
    flag_version: bool,
    flag_short: bool,
    flag_resource: Resource,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Resource { Core, Search, Graphql }

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
          &Resource::Core => write!(f, "core"),
          &Resource::Search => write!(f, "search"),
          &Resource::Graphql => write!(f, "graphql"),
        }
    }
}

#[derive(Debug)]
pub struct Options {
    pub frequency: u64,
    pub auth: AuthType,
    pub resource: Resource,
}

impl From<Arguments> for Options {
    fn from(item: Arguments) -> Self {
      if !item.flag_access_token.is_empty() {
            Options {
              resource: item.flag_resource.clone(),
              frequency: item.flag_frequency,
              auth: AuthType::Token(item.flag_access_token.clone())
            }
        }
        else if !item.flag_login.is_empty() {
            Options {
              resource: item.flag_resource.clone(),
              frequency: item.flag_frequency,
              auth: AuthType::Login {
                login: item.flag_login.clone(),
                password: item.flag_password.clone()
              }
            }
        }
        else {
            Options {
              resource: item.flag_resource.clone(),
              frequency: item.flag_frequency,
              auth: AuthType::Anonymos
            }
        }
    }
}

fn is_tty() -> bool {
    let tty = unsafe { libc::isatty(libc::STDOUT_FILENO as i32) } != 0;
    tty
}

pub fn get_options(usage: &str) -> Option<Options> {
    let version = format!("{}.{}.{}{}",
                     env!("CARGO_PKG_VERSION_MAJOR"),
                     env!("CARGO_PKG_VERSION_MINOR"),
                     env!("CARGO_PKG_VERSION_PATCH"),
                     option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));

    let args: Arguments = Docopt::new(usage)
                              .and_then(|d| Ok(d.version(Some(version))))
                              .and_then(|d| d.deserialize())
                              .unwrap_or_else(|e| e.exit());
    Some(args.into())
}