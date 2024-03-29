# xprompt

![build status](https://github.com/56quarters/xprompt/actions/workflows/rust.yml/badge.svg)

[![crates.io](https://img.shields.io/crates/v/xprompt.svg)](https://crates.io/crates/xprompt/)

Display a colorful Bash prompt.

`xprompt` is a CLI tool for displaying something that can be used as a `PS1` or
`PS2` value for your Bash prompt.

## Install

xprompt is a Rust project. If you want to install it, you'll need the Rust
toolchain. For more information about how to install Rust see https://www.rustup.rs/

After you have Rust installed, you can use Cargo to install xprompt.

```
cargo install --force xprompt
xprompt --help
```

## Usage

To use `xprompt` to set your Bash prompt, add the following to your `.bashrc` or
`.bash_profile` files in your home directory.

```
eval "$(xprompt init)"
```

Or, if you'd rather not `eval` things.

```
PS1="$(xprompt ps1)"
export PS1
PS2="$(xprompt ps2)"
export PS2
```

![xprompt screenshot](img/screenshot.png)

That's all you need!

## Limitations

Currently, xprompt is hardcoded to emit a prompt how I want. It's not configurable in
any way regarding colors, style, or information displayed. This is unlikely to change.
If you want a prompt that is more configurable, you should take a look at [Jess Fraz's dot
files](https://github.com/jessfraz/dotfiles/blob/663bc68ac49fe04d7a52ec410d3074d038dbe87f/.bash_prompt),
which the style of xprompt is based on, or the [Starship project](https://github.com/starship/starship).

## Source

The source code is available on GitHub at https://github.com/56quarters/xprompt

## Changes

Release notes for xprompt can be found in the [CHANGES.md](CHANGES.md) file.

## Development

xprompt uses Cargo for performing various development tasks.

To build Staccato:

```
$ cargo build
```

To run tests:

```
$ cargo test
```

To run benchmarks:

```
$ cargo bench
```

To build documentation:

```
$ cargo doc
```

## License

xprompt is available under the terms of the [GPL, version 3](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any
additional terms or conditions.

## Credit

The style of this prompt is based on [Jess Fraz's](https://github.com/jessfraz/) Bash
[dot files](https://github.com/jessfraz/dotfiles/blob/663bc68ac49fe04d7a52ec410d3074d038dbe87f/.bash_prompt).