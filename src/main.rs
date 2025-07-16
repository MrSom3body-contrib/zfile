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
    //declaring th standard output
    let mut stdout = io::stdout();
    //clearing the terminal and entrering a alternate screen
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    //declaring the terminal
    let mut terminal = Terminal::new(backend)?;

    //start in current dir
    let mut current_directory = std::env::current_dir()?;
    // index of curretn selected file
    let mut selected_file = 0;

    //having a bug becuase the terminal is not clearing the screen before drawing and so it
    //conflichts with ui
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
                .block(Block::default().title("zfile").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        // 2025 is the year for cyan xd
                        .fg(Color::Cyan),
                );
            f.render_stateful_widget(
                ui_list,
                f.area(),
                &mut ratatui::widgets::ListState::default().with_selected(Some(selected_file)),
            );
        })?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => {
                        if selected_file < get_entries(&current_directory).len().saturating_sub(1) {
                            selected_file += 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        if selected_file > 0 {
                            selected_file -= 1;
                        }
                    }
                    KeyCode::Char('h') => {
                        current_directory.pop();
                        selected_file = 0;
                    }
                    KeyCode::Char('l') => {
                        //only works for directories
                        //when its a file it will go back to the original directory
                        current_directory.push(entries[selected_file].clone());
                        selected_file = 1;
                    }
                    _ => {}
                }
            }
        }
    }
    //cleaning up so the terminal can be used again
    disable_raw_mode()?;
    //leaving the alternate screen
    execute!(io::stdout(), LeaveAlternateScreen)?;
    //return statement
    Ok(())
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
