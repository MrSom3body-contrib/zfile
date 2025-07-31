// for handling the terminal with user input
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
    // enabling raw mode
    enable_raw_mode()?;
    // declaring the standard output
    let mut stdout = io::stdout();
    // entering an alternate screen
    execute!(stdout, EnterAlternateScreen)?;
    // declaring the terminal
    let mut terminal = Some(init_terminal()?);

    // start in current dir
    let mut current_directory: PathBuf = std::env::current_dir()?;
    let root_dir: PathBuf = current_directory.clone();
    // index of currently selected file
    let mut selected_file: usize = 0;

    // search state
    let mut query: String = String::new();
    let mut in_search: bool = false;
    let mut fuzzy_mode: bool = false; // toggled by 'f'

    // reusable matcher
    let matcher = SkimMatcherV2::default();

    loop {
        // gather and filter entries
        let mut entries_raw = get_entries(&current_directory);

        // Apply filtering:
        // - No query => show all
        // - fuzzy_mode => keep items with a score, sort by score desc
        // - normal mode => case-insensitive contains
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
            // higher score first
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

        // keep selection in range if list shrank
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
                        // Quit
                        KeyCode::Char('q') => break,

                        // Enter search modes
                        KeyCode::Char('f') if !in_search => {
                            // Enter/keep search mode and toggle fuzzy
                            in_search = true;
                            fuzzy_mode = true;
                        }
                        KeyCode::Char('s') if !in_search => {
                            in_search = true;
                            fuzzy_mode = false;
                        }

                        // While in search, capture typing and editing
                        KeyCode::Esc if in_search => {
                            // leave search mode but keep the current filter
                            in_search = false;
                        }

                        KeyCode::Enter if in_search => {
                            // leave search mode but keep the current filter
                            in_search = false;
                        }
                        // While in search, capture typing and editing
                        KeyCode::Char(c) if in_search => {
                            // avoid stealing nav keys while searching by only handling in this arm
                            query.push(c);

                            selected_file = 0;
                        }
                        KeyCode::Backspace if in_search => {
                            query.pop();
                            selected_file = 0;
                        }

                        KeyCode::Char('ö') if in_search => {
                            // keep search results, exit typing mode
                            in_search = false;
                        }

                        // Navigation keys (only when NOT typing in the search bar)
                        KeyCode::Char('j') if !in_search => {
                            if !entries.is_empty()
                                && selected_file < entries.len().saturating_sub(1)
                            {
                                selected_file += 1;
                            }
                        }
                        KeyCode::Char('k') if !in_search => {
                            if selected_file > 0 {
                                selected_file -= 1;
                            }
                        }
                        KeyCode::Char('J') if !in_search => {
                            if !entries.is_empty() {
                                selected_file = entries.len().saturating_sub(1);
                            }
                        }
                        KeyCode::Char('K') if !in_search => {
                            selected_file = 0;
                        }
                        KeyCode::Char('h') if !in_search => {
                            current_directory.pop();
                            selected_file = 0;
                        }
                        KeyCode::Char('l') if !in_search => {
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
                        KeyCode::Char('H') if !in_search => {
                            // go to root_dir (fixed logic to avoid infinite loop)
                            while current_directory != root_dir {
                                current_directory.pop();
                            }
                            selected_file = 0;
                        }

                        _ => {}
                    }
                }
            }
        }
    }

    // cleaning up so the terminal can be used again
    disable_raw_mode()?;
    // leaving the alternate screen
    execute!(io::stdout(), LeaveAlternateScreen)?;
    // exit cleanly
    std::process::exit(0);
}

// get the entries in the directory and returns it as a vector of paths
fn get_entries(path: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(path)
        .unwrap_or_else(|_| fs::read_dir(".").unwrap())
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect()
}

// helper function for opening files with nvim and when closing nvim it returns to the parent directory
#[allow(unused)]
fn file_helper(path: &PathBuf) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Command::new("nvim")
        .arg(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?; // Waits for nvim to exit;
    Ok(())
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}
