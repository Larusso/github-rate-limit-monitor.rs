use grlm::AuthType;

use docopt::Docopt;

#[derive(Debug, Deserialize)]
struct Arguments {
    flag_login: String,
    flag_password: String,
    flag_access_token: String,
    flag_frequency: u64,
    flag_version: bool
}

impl Arguments {
    fn to_arguments(&self) -> Options {
        if !self.flag_access_token.is_empty() {
            Options {frequency: self.flag_frequency, auth: AuthType::Token(self.flag_access_token.clone())}
        }
        else if !self.flag_login.is_empty() {
            Options {frequency: self.flag_frequency, auth: AuthType::Login {login: self.flag_login.clone(), password: self.flag_password.clone()}}
        }
        else {
            Options {frequency: self.flag_frequency, auth: AuthType::Anonymos}
        }
    }
}

#[derive(Debug)]
pub struct Options {
    pub frequency: u64,
    pub auth: AuthType
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
    Some(args.to_arguments())
}