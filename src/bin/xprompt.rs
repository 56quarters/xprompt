// xprompt - Display a colorful Bash prompt
//
// Copyright 2020 Nick Pillitteri
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use ansi_term::{ANSIStrings, Color, Style};
use clap::{crate_version, Clap};
use git2::{Oid, Repository, Status};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::BTreeSet;
use std::env;
use std::fmt::{self, Display, Formatter, Write};

const TIMESTAMP: &str = r"\D{%Y-%m-%dT%H:%M:%S}";
const USER: &str = r"\u";
const WORKING_DIR: &str = r"\w";
const HOST: &str = r"\h";

/// Display a colorful Bash prompt
#[derive(Debug, Clap)]
#[clap(name = "xprompt", version = crate_version!())]
struct XpromptOptions {
    #[clap(subcommand)]
    mode: SubCommand,
}

#[derive(Debug, Clap)]
enum SubCommand {
    Ps1(Ps1Command),
    Ps2(Ps2Command),
    Vcs(VcsCommand),
}

/// Output a PS1 Bash prompt (standard prompt)
#[derive(Debug, Clap)]
struct Ps1Command {
    /// Path to xprompt itself if not installed on your PATH
    #[clap(long, default_value = "xprompt")]
    path: String,

    /// Prompt for user input (usually '$' or '#')
    #[clap(long, default_value = "$")]
    input: String,
}

/// Output a PS2 Bash prompt (continuation)
#[derive(Debug, Clap)]
struct Ps2Command;

/// Output version control information
#[derive(Debug, Clap)]
struct VcsCommand;

/// Potential states files in a Git repository or the repository itself could be in
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
        Display::fmt(self.val(), f)
    }
}

/// Wrapper around a string and associated style that implements `Display`.
///
/// Similar to `ANSIString` except Bash escape codes (`\[` and `\]`) are emitted
/// around non-printing characters to help Bash correctly calculate line length
/// for line editing purposes.
///
/// See the [ansi term issue](https://github.com/ogham/rust-ansi-term/issues/36)
/// for a more detailed description of the issue and why we're solving it here.
///
/// See also the [Bash man page](https://www.gnu.org/savannah-checkouts/gnu/bash/manual/bash.html#Controlling-the-Prompt).
struct BashString<'a> {
    style: Style,
    string: Cow<'a, str>,
}

impl<'a> BashString<'a> {
    fn new<S>(style: Style, string: S) -> Self
        where
            S: Into<Cow<'a, str>>,
    {
        BashString {
            style,
            string: string.into(),
        }
    }
}

impl<'a> Display for BashString<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\\[{}\\]{}\\[{}\\]",
            self.style.prefix(),
            self.string,
            self.style.suffix()
        )
    }
}

/// Wrapper around multiple `BashString` instances to be displayed together.
///
/// Similar to `ANSIStrings`, except that Bash escape codes (`\[` and `\]`) are
/// emitted around non-printing characters to help Bash correctly calculate
/// line length for line editing purposes.
///
/// Note that unlike `ANSIStrings`, this wrapper doesn't do anything clever
/// regarding only emitting reset sequences when needed. Instead, it assumes
/// all strings to be displayed differ only by color and therefore only emits
/// a single reset after the last string. We can make this tradeoff since we
/// know exactly how this object will be used.
///
/// See the [ansi term issue](https://github.com/ogham/rust-ansi-term/issues/36)
/// for a more detailed description of the issue and why we're solving it here.
///
/// See also the [Bash man page](https://www.gnu.org/savannah-checkouts/gnu/bash/manual/bash.html#Controlling-the-Prompt).
struct BashStrings<'a> {
    strings: &'a [BashString<'a>],
}

impl<'a> BashStrings<'a> {
    fn new(strings: &'a [BashString<'a>]) -> Self {
        BashStrings { strings }
    }
}

impl<'a> Display for BashStrings<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for s in self.strings {
            write!(f, "\\[{}\\]{}", s.style.prefix(), s.string)?;
        }

        if let Some(s) = self.strings.iter().last() {
            write!(f, "\\[{}\\]", s.style.suffix())?;
        }

        Ok(())
    }
}

/// Colors to use for writing output
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

/// Get the current working directory based on the `PWD` variable if possible
/// or fall back to using a function from the standard library
fn get_current_dir() -> Option<String> {
    env::var("PWD")
        .ok()
        // Environmental variable is missing (maybe someone unset it), so we fall
        // back to the standard library which will use platform specific logic and
        // potentially do a system call.
        .or_else(|| env::current_dir().ok().and_then(|p| p.to_str().map(|s| s.to_owned())))
}

/// Get the current git branch or commit if the current directory is a git repository
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

/// Get state of the current git repository (new files, modified, index, etc)
fn get_git_flags(repo: &mut Repository) -> BTreeSet<GitFlags> {
    let mut flags = BTreeSet::new();

    if let Ok(statuses) = repo.statuses(None) {
        for s in statuses.iter() {
            let status = s.status();

            if is_unversioned(status) {
                flags.insert(GitFlags::UNVERSIONED);
            }

            if is_working_tree_modified(status) {
                flags.insert(GitFlags::MODIFED);
            }

            if is_index_modified(status) {
                flags.insert(GitFlags::ADDED);
            }
        }
    }

    if is_stashed(repo) {
        flags.insert(GitFlags::STASHED);
    }

    flags
}

/// Is this status an unversioned file?
#[inline]
fn is_unversioned(status: Status) -> bool {
    (status & Status::WT_NEW) != Status::CURRENT
}

/// Is this status a modified file?
#[inline]
fn is_working_tree_modified(status: Status) -> bool {
    (status & (Status::WT_DELETED | Status::WT_MODIFIED | Status::WT_RENAMED | Status::WT_TYPECHANGE))
        != Status::CURRENT
}

/// Is this status a change to the index?
#[inline]
fn is_index_modified(status: Status) -> bool {
    (status
        & (Status::INDEX_DELETED
            | Status::INDEX_MODIFIED
            | Status::INDEX_NEW
            | Status::INDEX_RENAMED
            | Status::INDEX_TYPECHANGE))
        != Status::CURRENT
}

/// Are there any stashed changes in this repository?
#[inline]
fn is_stashed(repo: &mut Repository) -> bool {
    let stashed = Cell::new(false);

    let _ = repo.stash_foreach(|_a: usize, _b: &str, _c: &Oid| -> bool {
        stashed.set(true);
        // stop as soon as we determine that there's any stash
        false
    });

    stashed.get()
}

/// Write the colorized branch of the current git repository
fn write_git_branch(buf: &mut String, pallet: &Pallet, branch: &str) {
    // Use ANSIStrings here instead of BashStrings since we don't need to escape
    // non-printing characters when just writing from a bash function call (as opposed
    // to using output for setting PS1).
    let _ = write!(
        buf,
        "{branch}",
        branch = ANSIStrings(&[pallet.white.paint("on "), pallet.violet.paint(branch),])
    );
}

/// Write colorized information about the status of the current git repository
fn write_git_status(buf: &mut String, pallet: &Pallet, flags: &BTreeSet<GitFlags>) {
    let flag_str = flags.iter().map(|f| f.val()).collect::<Vec<&'static str>>().join("");

    // Use ANSIStrings here instead of BashStrings since we don't need to escape
    // non-printing characters when just writing from a bash function call (as opposed
    // to using output for setting PS1).
    let _ = write!(
        buf,
        "{flags}",
        flags = ANSIStrings(&[
            pallet.blue.paint(" ["),
            pallet.blue.paint(flag_str),
            pallet.blue.paint("]"),
        ])
    );
}

/// Write some colorized basic information to the given buffer
fn write_base_prompt(buf: &mut String, pallet: &Pallet) {
    let _ = write!(
        buf,
        "\\n{prompt}",
        prompt = BashStrings::new(&[
            BashString::new(pallet.cyan, TIMESTAMP),
            BashString::new(pallet.white, " as "),
            BashString::new(pallet.blue, USER),
            BashString::new(pallet.white, " at "),
            BashString::new(pallet.orange, HOST),
            BashString::new(pallet.white, " in "),
            BashString::new(pallet.green, WORKING_DIR),
        ])
    );
}

/// Write Bash code to call xprompt again in "VCS" mode.
///
/// PS1 is only set once per shell and we need version control information to
/// reflect the current state of a repository every time the prompt is displayed.
/// Thus, we don't emit the actual git status, just code to call xprompt again.
fn write_vcs_callback(buf: &mut String, path: &str) {
    let _ = write!(buf, " $({path} vcs)", path = path);
}

/// Write the '$' prompt for user input on a newline
fn write_command_prompt(buf: &mut String, pallet: &Pallet, input: &str) {
    let _ = write!(buf, "\\n{} ", BashString::new(pallet.white, input));
}

/// Get a string to represent PS1 (normal Bash prompt)
fn get_ps1(pallet: &Pallet, input: &str, path: &str) -> String {
    let mut buf = String::new();
    write_base_prompt(&mut buf, pallet);
    write_vcs_callback(&mut buf, path);
    write_command_prompt(&mut buf, pallet, input);
    buf
}

/// Get a string to represent PS2 (line continuation)
fn get_ps2(pallet: &Pallet) -> String {
    format!("{}", BashString::new(pallet.yellow, "-> "))
}

/// Get a string to represent version control information
fn get_vcs(pallet: &Pallet) -> String {
    let mut buf = String::new();

    let path = get_current_dir();
    let mut repo = path.and_then(|p| Repository::discover(p).ok());
    if let Some(ref mut r) = repo {
        let git_branch = get_git_branch(r);
        let git_flags = get_git_flags(r);

        if let Some(b) = git_branch {
            write_git_branch(&mut buf, &pallet, &b);
            if !git_flags.is_empty() {
                write_git_status(&mut buf, &pallet, &git_flags);
            }
        }
    }

    buf
}

fn main() {
    let opts = XpromptOptions::parse();
    let pallet = Pallet::default();

    let buf = match opts.mode {
        SubCommand::Ps1(c) => get_ps1(&pallet, &c.input, &c.path),
        SubCommand::Ps2(_c) => get_ps2(&pallet),
        SubCommand::Vcs(_c) => get_vcs(&pallet),
    };

    print!("{}", buf);
}
