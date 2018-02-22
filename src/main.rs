#[macro_use]
//Docop
extern crate serde_derive;
extern crate serde_json;
extern crate docopt;

use docopt::Docopt;
use std::{thread, time};

//Hyper
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate failure;
use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use hyper::header::{Headers, Authorization, UserAgent, Bearer};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use serde_json::Value;
use failure::Error;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

//Progressbar
extern crate indicatif;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

const USAGE: &'static str = "
grlm - github rate limit monitor

Usage:
  grlm (-l <user> -p <password> | -t <token>) [-f <frequency>]
  grlm --version
  grlm -h | --help

Options:
-l <user>, --login <user>                the github username
-p <password>, --password <password>     the user password
-t <token>, --access-token <token>        an github accesstoken
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

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Serialize, Deserialize, Debug)]
struct RateLimitResult {
    resources: GithubRateLimit,
    rate: RateLimit
}

#[derive(Serialize, Deserialize, Debug)]
struct GithubRateLimit {
    core: RateLimit,
    search: RateLimit,
    graphql: RateLimit
}

#[derive(Serialize, Deserialize, Debug)]
struct RateLimit {
    limit: u64,
    remaining: u64,
    reset: u64
}

struct Minutes(i64);

impl fmt::Display for Minutes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct Seconds(i64);

impl fmt::Display for Seconds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Seconds {
    pub fn to_minutes(&self) -> Minutes {
        Minutes(self.0 / 60)
    }
}

impl Minutes {
    pub fn to_seconds(&self) -> Seconds {
        Seconds(self.0 * 60)
    }
}

impl RateLimit {
    pub fn resets_in(&self) -> i64 {
        let utc_secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.reset as i64 - utc_secs
    }
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
    //println!("{:?}", args);

    if args.flag_version {
        println!("{:?}",version);
    } else {
        monitor(args)
    }
}

fn fetch_rate_limit(token : &String ) -> Result<RateLimitResult> {
    let mut core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
                 .connector(HttpsConnector::new(4, &handle)?)
                 .build(&handle);

    let uri = "https://api.github.com/rate_limit".parse()?;

    let mut req = hyper::Request::new(hyper::Method::Get, uri);
    req.headers_mut().set(Authorization(
       Bearer {
           token: token.to_owned()
       }
    ));
    req.headers_mut().set(UserAgent::new("curl/7.54.0"));

    let work = client.request(req).and_then(|res| {
        res.body().concat2().and_then(move |body| {
            let v: RateLimitResult = serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            })?;
            //println!("{:?}", v);
            Ok(v)
        })
    });
    let result = core.run(work)?;
    Ok(result)
}

fn fetch_rate_limit_fake(counter: &u64) -> Result<RateLimitResult> {
    let rem = match 5000 - counter * 34 {
        n @ 0 ... 5000 => n,
        _ => 0,
    };

    let resources = GithubRateLimit {
                                        core: RateLimit {limit: 5000, remaining: rem, reset: 0},
                                        graphql: RateLimit {limit: 5000, remaining: rem, reset: 0},
                                        search: RateLimit {limit: 5000, remaining: rem, reset: 0}
                                    };

    let r = RateLimitResult {rate: RateLimit {limit: 5000, remaining: rem, reset: 0}, resources: resources};
    Ok(r)
}

fn monitor(ref args : Args) {
    let f = args.flag_frequency;
    let bar = ProgressBar::new(5000);
    bar.set_style(ProgressStyle::default_bar()
    .template(&format!("{{prefix:.bold}} {{pos}} {{wide_bar:.{}}} of {{len}} {{msg.{}}} ", "yellow", "yellow"))
    .progress_chars(" \u{15E7}\u{FF65}"));

    bar.set_prefix("Requests");
    let mut counter = 0;
    loop {
        match fetch_rate_limit(&args.flag_access_token) {
            Ok(r) => {
                let rate_color = match (r.rate.remaining as f64) / (r.rate.limit as f64) {
                    x if x <= 0.08 => "red",
                    x if x <= 0.5 => "yellow",
                    _ => "green"
                };

                let message_color = match r.rate.resets_in() {
                    x if x < 120 => "green",
                    _ => "white"
                };

                bar.set_message(&format!("{}",r.rate.resets_in()));
                bar.set_style(ProgressStyle::default_bar()
                .template(&format!("{{prefix:.bold}} {{pos:.{}}} {{wide_bar:.{}}} of {{len}} resets in {{msg:.{}}} ", rate_color, "yellow", message_color))
                .progress_chars(" \u{15E7}\u{FF65}"));
                bar.set_position(r.rate.limit - r.rate.remaining);
            },
            Err(e) => println!("Error {}", e),
        }
        counter += 1;
        //
        let ten_millis = time::Duration::from_secs(f);
        thread::sleep(ten_millis)
    }
}