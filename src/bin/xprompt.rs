//
//
//
//

use ansi_term::{ANSIStrings, Color, Style};
use chrono::Local;
use clap::{crate_version, Clap};
use git2::{Repository, Status};
use std::collections::BTreeSet;
use std::env;
use std::fmt::{self, Display, Formatter, Write};

///
#[derive(Debug, Clap)]
#[clap(name = "xprompt", version = crate_version!())]
struct PrompterOptions {
    /// Prints the PS1 prompt
    #[clap(long)]
    ps1: bool,

    /// Prints the PS2 prompt
    #[clap(long)]
    ps2: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum GitFlags {
    UNVERSIONED,
    MODIFED,
    ADDED,
    STASHED,
}

impl GitFlags {
    fn val(&self) -> &'static str {
        match self {
            Self::UNVERSIONED => "?",
            Self::MODIFED => "!",
            Self::ADDED => "+",
            Self::STASHED => "$",
        }
    }
}

impl Display for GitFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.val().fmt(f)
    }
}

impl<'a> Into<&'a str> for GitFlags {
    fn into(self) -> &'a str {
        self.val()
    }
}

impl<'a> Into<&'a str> for &'a GitFlags {
    fn into(self) -> &'a str {
        self.val()
    }
}

#[allow(dead_code)]
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
    env::var("PWD")
        .ok()
        // Environmental variable is missing (maybe someone unset it), so we fall
        // back to the standard library which will use platform specific logic and
        // potentially do a system call.
        .or_else(|| env::current_dir().ok().and_then(|p| p.to_str().map(|s| s.to_owned())))
}

fn get_relative_dir(home: &Option<String>, current: &Option<String>) -> Option<String> {
    if let Some(h) = home {
        if let Some(c) = current {
            return Some(if c.starts_with(h) {
                c.replace(h, "~")
            } else {
                c.to_owned()
            });
        }
    }

    None
}

fn get_host() -> Option<String> {
    gethostname::gethostname().to_str().map(|s| s.to_string())
}

fn get_git_branch(repo: &Repository) -> Option<String> {
    if let Ok(r) = repo.head() {
        if r.is_branch() {
            r.shorthand().map(|s| s.to_owned())
        } else {
            r.peel_to_commit().ok().map(|c| c.id().to_string())
        }
    } else {
        None
    }
}

fn get_git_flags(repo: &Repository) -> BTreeSet<GitFlags> {
    let mut out = BTreeSet::new();
    if let Ok(statuses) = repo.statuses(None) {
        for s in statuses.iter() {
            let status = s.status();

            if is_unversioned(status) {
                out.insert(GitFlags::UNVERSIONED);
            }

            if is_working_tree_modified(status) {
                out.insert(GitFlags::MODIFED);
            }

            if is_index_modified(status) {
                out.insert(GitFlags::ADDED);
            }
        }
    }

    out
}

fn is_unversioned(status: Status) -> bool {
    (status & Status::WT_NEW) != Status::CURRENT
}

fn is_working_tree_modified(status: Status) -> bool {
    (status & (Status::WT_DELETED | Status::WT_MODIFIED | Status::WT_RENAMED | Status::WT_TYPECHANGE))
        != Status::CURRENT
}

fn is_index_modified(status: Status) -> bool {
    (status
        & (Status::INDEX_DELETED
            | Status::INDEX_MODIFIED
            | Status::INDEX_NEW
            | Status::INDEX_RENAMED
            | Status::INDEX_TYPECHANGE))
        != Status::CURRENT
}

fn write_git_branch() {
    todo!()
}

fn write_git_status() {
    todo!()
}

fn main() {
    let _opts = PrompterOptions::parse();
    let path = get_current_dir();
    let home = get_home();
    let user = get_user();
    let timestamp = get_timestamp();
    let host = get_host();
    let relative = get_relative_dir(&home, &path);

    let repo = path.and_then(|p| Repository::discover(p).ok());
    let mut git_branch = None;
    let mut git_flags = BTreeSet::new();

    if let Some(r) = &repo {
        git_branch = get_git_branch(r);
        git_flags = get_git_flags(r);
    }

    let pallet = Pallet::default();

    let mut buf = String::new();
    let _ = write!(
        &mut buf,
        "{prompt}",
        prompt = ANSIStrings(&[
            pallet.cyan.paint(timestamp),
            pallet.white.paint(" as "),
            pallet.blue.paint(user.unwrap_or("[unknown]".to_owned())),
            pallet.white.paint(" at "),
            pallet.orange.paint(host.unwrap_or("[unknown]".to_owned())),
            pallet.white.paint(" in "),
            pallet.green.paint(relative.unwrap_or("[unknown]".to_owned()))
        ])
    );

    if let Some(b) = git_branch {
        let _ = write!(
            &mut buf,
            "{branch}",
            branch = ANSIStrings(&[pallet.white.paint(" on "), pallet.violet.paint(b),])
        );

        if !git_flags.is_empty() {
            let flags = git_flags
                .iter()
                .map(|f| f.val())
                .collect::<Vec<&'static str>>()
                .join("");

            let _ = write!(
                &mut buf,
                "{flags}",
                flags = ANSIStrings(&[
                    pallet.blue.paint(" ["),
                    pallet.blue.paint(flags),
                    pallet.blue.paint("]"),
                ])
            );
        }
    }

    print!("{}", buf);
}
