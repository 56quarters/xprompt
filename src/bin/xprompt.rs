//
//
//
//

use std::env;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process;
use std::time;

use ansi_term::{Color, Style};
use chrono::{DateTime, Local};
use clap::Clap;
use git2::Repository;

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
    yellow: Style,
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
#[clap(name = "xprompt")]
struct PrompterOptions {}

fn get_timestamp() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn get_user() -> Option<String> {
    env::var("USER").ok()
}

fn get_home() -> Option<String> {
    env::var("HOME").ok()
}

fn get_current_dir() -> Option<String> {
    env::var("PWD").ok().or_else(|| {
        env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_owned()))
    })
}

fn get_relative_dir(home: &Option<String>, current: &Option<String>) -> Option<String> {

    if let Some(h) = home {
        if let Some(c) = current {
            return Some(if c.starts_with(h) {
                c.replace(h, "~")
            } else {
                c.to_owned()
            })
        }
    }

    None
}

fn get_host() -> Option<String> {
    gethostname::gethostname().to_str().map(|s| s.to_string())
}

fn main() {
    let _opts = PrompterOptions::parse();
    let path = get_current_dir();
    let home = get_home();
    let user = get_user();
    let timestamp = get_timestamp();
    let host = get_host();
    let relative = get_relative_dir(&home, &path);

    // TODO: obviously this should be optional
    //let repo = Repository::discover(&path).ok();

    /*
    let statuses = repo.statuses(None).unwrap();
    let state = repo.state();
    for s in statuses.iter() {
        //println!("status {}: {:?}", s.path().unwrap_or(""), s.status());
    }
     */

    let pallet = Pallet::default();

    print!(
        "{timestamp} {as_} {user} {at} {host} {in_} {dir} {on} {git}",
        timestamp = pallet.cyan.paint(timestamp),
        as_ = pallet.white.paint("as"),
        user = pallet.blue.paint(user.unwrap_or("[unknown]".to_owned())),
        at = pallet.white.paint("at"),
        host = pallet.orange.paint(host.unwrap_or("[unknown]".to_owned())),
        in_ = pallet.white.paint("in"),
        dir = pallet
            .green
            .paint(relative.unwrap_or("[unknown]".to_owned())),
        on = pallet.white.paint("in"),
        git = pallet.violet.paint("[git!]"),
    )
}
