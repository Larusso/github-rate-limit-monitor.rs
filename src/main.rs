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

fn monitor(ref args : Args) {
    let f = args.flag_frequency;
    let bar = ProgressBar::new(5000);
    bar.set_style(ProgressStyle::default_bar()
    //.template(&format!("{{prefix:.bold}}▕{{bar:.{}}}▏{{msg}}", "yellow"))
    .progress_chars("█▛▌▖  "));
    // bar.set_style(ProgressStyle::default_bar()
    // .template(&format!("{{prefix:.bold}}▕{{bar:.{}}}▏{{msg}}", "yellow"))

    bar.set_prefix("Requests");

    loop {
        match fetch_rate_limit(&args.flag_access_token) {
            Ok(r) => bar.set_position(r.rate.limit - r.rate.remaining),
            Err(e) => println!("Error {}", e),
        }
        //
        let ten_millis = time::Duration::from_secs(f);
        thread::sleep(ten_millis)
    }
}