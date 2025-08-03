// for handling the terminal with user input
mod file_manipulation;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::process::{Command, Stdio};
// for the ui components
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
};
// for getting the data from the file system
use std::{fs, io, path::PathBuf};

// fuzzy matching
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Some(init_terminal()?);

    let mut current_directory: PathBuf = std::env::current_dir()?;
    let root_dir: PathBuf = current_directory.clone();
    let mut selected_file: usize = 0;

    let mut query: String = String::new();
    let mut in_search: bool = false;
    let mut fuzzy_mode: bool = false;

    let mut move_buffer: String = String::new();
    let mut rename_buffer: String = String::new();
    let mut in_rename: bool = false;
    let mut in_move: bool = false;
    let mut double_auth_delete = false;

    let matcher = SkimMatcherV2::default();

    loop {
        let mut entries_raw = get_entries(&current_directory);
        let entries: Vec<PathBuf> = if query.is_empty() {
            entries_raw
        } else if fuzzy_mode {
            let q = query.clone();
            let mut scored: Vec<(PathBuf, i64)> = entries_raw
                .drain(..)
                .filter_map(|p| {
                    let name = p
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    matcher.fuzzy_match(&name, &q).map(|score| (p, score))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            scored.into_iter().map(|(p, _)| p).collect()
        } else {
            let q = query.to_lowercase();
            entries_raw
                .into_iter()
                .filter(|p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&q)
                })
                .collect()
        };

        if entries.is_empty() {
            selected_file = 0;
        } else if selected_file >= entries.len() {
            selected_file = entries.len().saturating_sub(1);
        }

        if let Some(ref mut term) = terminal {
            term.draw(|f| {
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(f.area());

                let nav_column = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(layout[0]);

                let title = if fuzzy_mode {
                    "Search (Fuzzy): type to filter, Esc to exit"
                } else if in_search {
                    "Search: type to filter, Esc to exit"
                } else {
                    "Search (press 'f' for fuzzy, '/' for normal)"
                };

                let search_paragraph = ratatui::widgets::Paragraph::new(query.as_str())
                    .block(Block::default().title(title).borders(Borders::ALL));
                f.render_widget(search_paragraph, nav_column[0]);

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
                    .block(Block::default().title("files").borders(Borders::ALL))
                    .highlight_style(Style::default().fg(Color::Cyan));

                let mut list_state = ratatui::widgets::ListState::default();
                if !entries.is_empty() {
                    list_state.select(Some(selected_file));
                }
                f.render_stateful_widget(ui_list, nav_column[1], &mut list_state);

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
                f.render_widget(preview, layout[1]);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,

                        KeyCode::Char('f') if !in_search && !in_rename && !in_move => {
                            in_search = true;
                            fuzzy_mode = true;
                        }
                        KeyCode::Char('s') if !in_search && !in_rename && !in_move => {
                            in_search = true;
                            fuzzy_mode = false;
                        }
                        KeyCode::Esc if in_search && !in_rename && !in_move => {
                            in_search = false;
                        }
                        KeyCode::Enter if in_search && !in_rename && !in_move => {
                            in_search = false;
                        }
                        KeyCode::Char('ö') if in_search && !in_rename && !in_move => {
                            selected_file = 0;
                            in_search = false;
                            #[allow(unused)]
                            file_helper(&entries[selected_file]);
                        }
                        KeyCode::Char(c) if in_search && !in_rename && !in_move => {
                            query.push(c);
                            selected_file = 0;
                        }
                        KeyCode::Backspace if in_search && !in_rename && !in_move => {
                            query.pop();
                            selected_file = 0;
                        }
                        KeyCode::Char('j') if !in_search && !in_rename && !in_move => {
                            if !entries.is_empty()
                                && selected_file < entries.len().saturating_sub(1)
                            {
                                selected_file += 1;
                            }
                        }
                        KeyCode::Char('k')
                            if selected_file > 0 && !in_search && !in_rename && !in_move =>
                        {
                            selected_file -= 1;
                        }
                        KeyCode::Char('J') if !in_search && !in_rename && !in_move => {
                            if !entries.is_empty() {
                                selected_file = entries.len().saturating_sub(1);
                            }
                        }
                        KeyCode::Char('K') if !in_search && !in_rename && !in_move => {
                            selected_file = 0;
                        }
                        KeyCode::Char('h') if !in_search && !in_rename && !in_move => {
                            current_directory.pop();
                            selected_file = 0;
                        }
                        KeyCode::Char('l') if !in_search && !in_rename && !in_move => {
                            if let Some(pointer_to_file) = entries.get(selected_file) {
                                if pointer_to_file.is_dir() {
                                    current_directory = pointer_to_file.clone();
                                    selected_file = 0;
                                } else if pointer_to_file.is_file() {
                                    if file_helper(&pointer_to_file).is_ok() {
                                        terminal = Some(init_terminal()?);
                                        current_directory = pointer_to_file
                                            .parent()
                                            .map(PathBuf::from)
                                            .unwrap_or(current_directory.clone());
                                        selected_file = 0;
                                    }
                                }
                            }
                        }
                        KeyCode::Char('H') if !in_search && !in_rename && !in_move => {
                            while current_directory != root_dir {
                                current_directory.pop();
                            }
                            selected_file = 0;
                        }

                        KeyCode::Char('d') if !in_search && !in_rename && !in_move => {
                            double_auth_delete = true;
                        }
                        KeyCode::Char('y') if double_auth_delete => {
                            if let Some(file) = entries.get(selected_file) {
                                if let Err(err) = file_manipulation::delete_file(file) {
                                    println!("Failed to delete file: {}", err);
                                }
                            }
                            double_auth_delete = false;
                        }
                        KeyCode::Char('n') if double_auth_delete => {
                            double_auth_delete = false;
                        }

                        KeyCode::Char('m') if !in_search && !in_rename => {
                            in_move = true;
                            move_buffer.clear();
                        }
                        KeyCode::Char(c) if in_move => {
                            move_buffer.push(c);
                        }
                        KeyCode::Enter if in_move => {
                            if let Some(file) = entries.get(selected_file) {
                                if let Err(err) = file_manipulation::move_file(file, &move_buffer) {
                                    println!("Failed to move file: {}", err);
                                }
                            }
                            in_move = false;
                            move_buffer.clear();
                        }

                        KeyCode::Char('r') if !in_search && !in_move => {
                            in_rename = true;
                            rename_buffer.clear();
                        }
                        KeyCode::Char(c) if in_rename => {
                            rename_buffer.push(c);
                        }
                        KeyCode::Enter if in_rename => {
                            if let Some(file) = entries.get(selected_file) {
                                if let Err(err) =
                                    file_manipulation::rename_file(file, &rename_buffer)
                                {
                                    println!("Failed to rename file: {}", err);
                                }
                            }
                            in_rename = false;
                            rename_buffer.clear();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    std::process::exit(0);
}

fn get_entries(path: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(path)
        .unwrap_or_else(|_| fs::read_dir(".").unwrap())
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect()
}

#[allow(unused)]
fn file_helper(path: &PathBuf) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Command::new("nvim")
        .arg(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(())
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}
