# zfile

A simple file manager written in Rust. The aim of zfile is to be dead simple but powerful because of the use of my own motions that helps navigating fastly through the file system.

## Goals

- ~~opening files with nvim~~
- ~~using own motions to navigate (j, k, h, l), (J,K)~~
- ~~fuzzy finder and normal search~~
- ~~preview file~~ but not optimized yet (large files)
- rename, delete, copy, move
- sort by name, size, date
- git branch history
- picture preview

## Installation

Its not finished yet so you need to compile it yourself.

```bash
cargo build --release
```

```bash
./target/release/zfile
```

## License

zfile is licensed under the MIT license.

## Credits

its based on crossterm,ratatui

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)
