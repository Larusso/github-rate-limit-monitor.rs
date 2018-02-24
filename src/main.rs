#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate docopt;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate failure;
extern crate indicatif;

use docopt::Docopt;
use std::{thread, time};
use std::time::{Duration, Instant};

//Progressbar

use indicatif::ProgressDrawTarget;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

mod grlm;

use grlm::{AuthType, RateLimitResult, GithubRateLimit, RateLimit};

const USAGE: &'static str = "
grlm - github rate limit monitor

Usage:
  grlm [(-l <user> -p <password> | -t <token>)] [-f <frequency>]
  grlm --version
  grlm -h | --help

Options:
  -l <user>, --login <user>                the github username
  -p <password>, --password <password>     the user password
  -t <token>, --access-token <token>       an github accesstoken
  -f <frequency>, --frequency <frequency>  refresh freqency [default: 10]
  -V, --version                            print version
  -h, --help                               show this help message and exit
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_login: String,
    flag_password: String,
    flag_access_token: String,
    flag_frequency: u64,
    flag_version: bool
}

impl Args {
    fn to_arguments(&self) -> Arguments {
        if !self.flag_access_token.is_empty() {
            Arguments {frequency: self.flag_frequency, auth: AuthType::Token(self.flag_access_token.clone())}
        }
        else if !self.flag_login.is_empty() {
            Arguments {frequency: self.flag_frequency, auth: AuthType::Login {login: self.flag_login.clone(), password: self.flag_password.clone()}}
        }
        else {
            Arguments {frequency: self.flag_frequency, auth: AuthType::Anonymos}
        }
    }
}

#[derive(Debug)]
struct Arguments {
    frequency: u64,
    auth: AuthType
}

fn main() {
    let version = format!("{}.{}.{}{}",
                     env!("CARGO_PKG_VERSION_MAJOR"),
                     env!("CARGO_PKG_VERSION_MINOR"),
                     env!("CARGO_PKG_VERSION_PATCH"),
                     option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));

    let args: Args = Docopt::new(USAGE)
                      .and_then(|d| d.deserialize())
                      .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("grlm {}",version);
    } else {
        monitor(args.to_arguments())
    }
}

fn monitor(ref args : Arguments) {
    let f = args.frequency;
    let initial_length = match args.auth {
        AuthType::Anonymos => 60,
        _ => 5000,
    };

    let bar = ProgressBar::new(initial_length);
    bar.set_draw_target(ProgressDrawTarget::stderr_nohz());
    bar.set_style(ProgressStyle::default_bar()
    .template(&format!("{{prefix:.bold}} {{pos}} {{wide_bar:.{}}} of {{len}} {{msg.{}}} ", "yellow", "yellow"))
    .progress_chars(" \u{15E7}\u{FF65}"));

    bar.set_prefix("Requests");
    let tick = Duration::from_secs(f);
    let mut instant = Instant::now();
    let mut first_run = true;
    loop {
        if first_run || instant.elapsed() >= tick
        {
            match grlm::fetch_rate_limit(&args.auth) {
                Ok(r) => {
                    //println!("{:?}", r);
                    let rate_color = match (r.rate.remaining as f64) / (r.rate.limit as f64) {
                        x if x <= 0.08 => "red",
                        x if x <= 0.5 => "yellow",
                        _ => "green"
                    };

                    let message_color = match r.rate.resets_in() {
                        x if x < 120 => "green",
                        _ => "white"
                    };

                    bar.set_length(r.rate.limit);
                    bar.set_message(&format!("{}",r.rate.resets_in()));
                    bar.set_style(ProgressStyle::default_bar()
                        .template(&format!("{{prefix:.bold}} {{pos:.{}}} {{wide_bar:.{}}} of {{len}} resets in {{msg:.{}}} ", rate_color, "yellow", message_color))
                        .progress_chars(" \u{15E7}\u{FF65}"));
                    bar.set_position(r.rate.limit - r.rate.remaining);
                },
                Err(e) => println!("Error {}", e),
            }
            instant = Instant::now();
        }
        first_run = false;

        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis)
    }
}