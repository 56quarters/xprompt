//
//
//
//

use std::fmt::Debug;
use std::path::PathBuf;
use std::process;
use std::time;

use chrono::{DateTime, Local};
use clap::Clap;
use git2::Repository;
use ansi_term::{Style, Color};

struct Pallet {
    black: Style,
    blue: Style,
    cyan: Style,
    green: Style,
    orange: Style,
    purple: Style,
    red: Style,
    violet: Style,
    white: Style,
    yellow: Style
}


impl Default for Pallet {
    fn default() -> Self {
        Pallet {
            // https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
            black: Color::Fixed(0).bold(),
            blue: Color::Fixed(33).bold(),
            cyan: Color::Fixed(37).bold(),
            green: Color::Fixed(64).bold(),
            orange: Color::Fixed(166).bold(),
            purple: Color::Fixed(125).bold(),
            red: Color::Fixed(124).bold(),
            violet: Color::Fixed(61).bold(),
            white: Color::Fixed(15).bold(),
            yellow: Color::Fixed(136).bold(),
        }
    }
}


#[derive(Debug, Clap)]
#[clap(name = "prompter")]
struct PrompterOptions {
    #[clap(name = "PATH", parse(from_os_str))]
    path: Option<PathBuf>
}

fn get_timestamp() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn get_user() -> String {
    std::env::var("USER").unwrap_or("unknown".to_owned())
}

fn get_home() -> String {
    std::env::var("HOME").unwrap_or("".to_owned())
}

fn get_host() -> String {
    gethostname::gethostname().to_str().map(|s| s.to_string()).unwrap_or("unknown".to_owned())
}

fn main() {
    let opts = PrompterOptions::parse();
    let git_path = opts.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let cur_path = opts.path.clone().or_else(|| std::env::current_dir().ok())
        .and_then(|p| p.to_str().map(|s| s.to_owned()))
        .unwrap_or_else(|| "~".to_owned());

    // TODO: obviously this should be optional
    let repo = match Repository::discover(&git_path) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("prompter: {:?} doesn't appear to be a git repository: {}", &git_path, e);
            process::exit(1);
        }
    };

    let statuses = repo.statuses(None).unwrap();

    let state = repo.state();

    for s in statuses.iter() {
        //println!("status {}: {:?}", s.path().unwrap_or(""), s.status());
    }

    let pallet = Pallet::default();

    println!(
        "{ts} {as_} {user} {at} {host} {in_} {dir} {on} {git}",
        ts=pallet.cyan.paint(get_timestamp()),
        as_=pallet.white.paint("as"),
        user=pallet.blue.paint(get_user()),
        at=pallet.white.paint("at"),
        host=pallet.orange.paint(get_host()),
        in_=pallet.white.paint("in"),
        dir=pallet.green.paint(cur_path),
        on=pallet.white.paint("in"),
        git=pallet.violet.paint("[git!]"),
    )

    //println!("state: {:?}", state);

    //println!("Hello, world!");
}
