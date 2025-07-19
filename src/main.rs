//for handling the terminal with user input
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::path::Path;
use std::process::{Command, Stdio};
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
    //enabling raw mode
    enable_raw_mode()?;
    //declaring th standard output
    let mut stdout = io::stdout();
    //clearing the terminal and entrering a alternate screen
    //for debugging
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
            let display_split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(f.area()); // draw the ui components
            // declaring each item
            let items: Vec<ListItem> = entries
                .iter()
                .map(|entry| {
                    let name = entry.file_name().unwrap_or_default().to_string_lossy();
                    let display_name = if entry.is_dir() {
                        format!("{}/", name)
                    } else {
                        name.to_string()
                    };
                    ListItem::new(display_name)
                })
                .collect();
            let ui_list = List::new(items)
                .block(Block::default().title("zfile").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        // 2025 is the year for cyan xd
                        .fg(Color::Cyan),
                );

            let mut list_state = ratatui::widgets::ListState::default();
            list_state.select(Some(selected_file));
            f.render_stateful_widget(ui_list, display_split[0], &mut list_state);

            let preview_content = if let Some(entry) = entries.get(selected_file) {
                if entry.is_file() {
                    fs::read_to_string(entry)
                        .unwrap_or_else(|_| "[Could not read file]".to_string())
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            let preview = ratatui::widgets::Paragraph::new(preview_content)
                .block(Block::default().title("Preview").borders(Borders::ALL))
                .wrap(ratatui::widgets::Wrap { trim: true });

            f.render_widget(preview, display_split[1]);
        })?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => {
                        //OPENING NEXT FILE/DIR HOTKEY
                        if selected_file < get_entries(&current_directory).len().saturating_sub(1) {
                            selected_file += 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        //GOING DOWN HOTKEY
                        if selected_file > 0 {
                            selected_file -= 1;
                        }
                    }
                    KeyCode::Char('h') => {
                        //GOING UP HOTKEY
                        current_directory.pop();
                        selected_file = 0;
                    }
                    KeyCode::Char('l') => {
                        //PARENT DIRECTORY HOTKEY
                        if let Some(pointer_to_file) = entries.get(selected_file) {
                            if pointer_to_file.is_dir() {
                                current_directory = pointer_to_file.clone();
                                selected_file = 0;
                            } else if pointer_to_file.is_file() {
                                if let Ok(new_dir) = file_helper(&pointer_to_file) {
                                    current_directory = new_dir;
                                    selected_file = 0;
                                }
                            }
                        }
                    }
                    KeyCode::Char('J') => {
                        if selected_file < get_entries(&current_directory).len().saturating_sub(1) {
                            selected_file = get_entries(&current_directory).len().saturating_sub(1);
                        }
                    }
                    KeyCode::Char('K') => {
                        if selected_file <= get_entries(&current_directory).len().saturating_sub(1)
                        {
                            selected_file = 0;
                        }
                    }
                    //dont need a hotkey for showing preview im gonna do it that it shows intantly
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
    //Solution to the issue when exiting nvim and exiting the program
    std::process::exit(0);
}
//get the entries in the directory and returns it as a string, i got this from chatgpt dont know how to explain it
// if the entry is a directory append "/" to it
fn get_entries(path: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(path)
        .unwrap_or_else(|_| fs::read_dir(".").unwrap())
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect()
}

//helper function for opening files with nvim and when closing nvim it returns to the parent directory
#[allow(unused)]
fn file_helper(path: &PathBuf) -> io::Result<PathBuf> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Command::new("nvim")
        .arg(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?; // Waits for nvim to exit;

    let new_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    // Re-enter TUI

    //i know its not the best solution but it works
    main()?;
    Ok(new_dir)
}
