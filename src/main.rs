//for handling the terminal with user input
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
// for the ui components
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
};
//for getting the data from the file system
use std::{fs, io, path::PathBuf};

fn main() -> Result<(), io::Error> {
    println!("testing");
    //declaring
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    //start in current dir
    let mut current_directory = std::env::current_dir()?;
    // index of curretn selected file
    let _selected_file = 0;

    loop {
        let entries = get_entries(&mut current_directory);

        terminal.draw(|f| {
            // draw the ui components
            // declaring each item
            let items: Vec<ListItem> = entries
                .iter()
                .map(|entry| ListItem::new(entry.as_str()))
                .collect();

            let ui_list = List::new(items)
                .block(Block::default().title("Files").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        // 2025 is the year for cyan xd
                        .fg(Color::Cyan),
                );
            f.render_stateful_widget(
                ui_list,
                f.area(),
                &mut ratatui::widgets::ListState::default().with_selected(Some(_selected_file)),
            );
        })?;
    }
}
//get the entries in the directory and returns it as a string, i got this from chatgpt dont know how to explain it
// if the entry is a directory append "/" to it
fn get_entries(path: &PathBuf) -> Vec<String> {
    fs::read_dir(path)
        .unwrap_or_else(|_| fs::read_dir(".").unwrap()) // fallback to current dir if error
        .filter_map(Result::ok) // skip unreadable entries
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                // Append "/" to directories
                format!("{}/", path.file_name().unwrap().to_string_lossy())
            } else {
                path.file_name().unwrap().to_string_lossy().to_string()
            }
        })
        .collect()
}
