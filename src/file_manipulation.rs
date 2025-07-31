use std::fs;
use std::io;
use std::path::PathBuf;
fn rename_file(from: &PathBuf, to: &PathBuf) -> io::Result<()> {
    fs::rename(from, to)
}
fn delete_file() {}
fn copy_file() {}
fn move_file() {}
