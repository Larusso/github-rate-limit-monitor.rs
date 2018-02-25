mod github;

pub mod cli;

pub use self::github::{AuthType, RateLimitResult, GithubRateLimit, RateLimit, fetch_rate_limit};

use indicatif::ProgressDrawTarget;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::time::{Duration, Instant};

pub fn monitor(ref args : cli::Options) {
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
            match fetch_rate_limit(&args.auth) {
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

        // let ten_millis = time::Duration::from_millis(10);
        // thread::sleep(ten_millis)
    }
}