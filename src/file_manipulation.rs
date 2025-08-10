use std::fs;
use std::io;
use std::path::PathBuf;

pub fn create_file(parent: &PathBuf, name: &str) -> io::Result<()> {
    let new_path = parent.join(name);
    fs::File::create(new_path)?;
    Ok(())
}
pub fn rename_file(old_path: &PathBuf, new_name: &str) -> io::Result<()> {
    let parent = old_path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Could not determine parent directory")
    })?;
    let new_path = parent.join(new_name);
    fs::rename(old_path, new_path)?;
    Ok(())
}

pub fn move_file(old_path: &PathBuf, new_path_str: &str) -> io::Result<()> {
    // in work
    let new_path = PathBuf::from(new_path_str);
    if new_path.is_dir() {
        let file_name = old_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid file name"))?;
        let destination = new_path.join(file_name);
        fs::rename(old_path, destination)?;
    } else {
        fs::rename(old_path, new_path)?;
    }
    Ok(())
}

pub fn delete_file(path: &PathBuf) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}
