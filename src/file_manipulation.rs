use std::fs;
use std::io;
use std::path::PathBuf;
fn rename_file(from: &PathBuf, to: &PathBuf) -> io::Result<()> {
    fs::rename(from, to)
}
fn delete_file(path: &PathBuf) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}
fn move_file(from: &PathBuf, to: &PathBuf) -> io::Result<()> {
    let filename = from.file_name().unwrap();
    let mut target = to.clone();
    target.push(filename);
    fs::rename(from, target)
}
