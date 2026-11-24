use poker_agent::opponent::{OpponentBook, Population};
use poker_agent::session::{AdvisorSession, TableUpdate};
use poker_agent::table::TableSnapshot;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tauri::State;

const BOOK_FILE_NAME: &str = "opponents.json";

pub struct AdvisorState {
    session: Mutex<AdvisorSession>,
    book_path: Option<PathBuf>,
}

impl AdvisorState {
    /// Loads the opponent stats saved in `data_dir`, or starts with none when
    /// there is no directory or no file. A file that cannot be read is moved
    /// aside as `opponents.json.unreadable` so the next save does not erase it.
    pub fn load(data_dir: Option<PathBuf>) -> AdvisorState {
        let book_path = data_dir.map(|dir| dir.join(BOOK_FILE_NAME));
        let book = book_path
            .as_deref()
            .map(load_book_or_set_aside)
            .unwrap_or_default();
        AdvisorState {
            session: Mutex::new(AdvisorSession::new(book)),
            book_path,
        }
    }

    fn session(&self) -> Result<MutexGuard<'_, AdvisorSession>, String> {
        self.session
            .lock()
            .map_err(|_| "the advisor stopped after an earlier error; restart the app".to_string())
    }

    fn save(&self, book: &OpponentBook) -> Result<(), String> {
        let Some(path) = &self.book_path else {
            return Ok(());
        };
        save_book(path, book).map_err(|error| {
            format!(
                "could not save opponent stats to {}: {error}",
                path.display()
            )
        })
    }
}

fn load_book_or_set_aside(path: &Path) -> OpponentBook {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return OpponentBook::default(),
        Err(error) => {
            eprintln!("could not read {}: {error}", path.display());
            return OpponentBook::default();
        }
    };
    match serde_json::from_str(&text) {
        Ok(book) => book,
        Err(error) => {
            eprintln!("could not parse {}: {error}", path.display());
            let aside = path.with_extension("json.unreadable");
            if let Err(error) = fs::rename(path, &aside) {
                eprintln!("could not move {} aside: {error}", path.display());
            }
            OpponentBook::default()
        }
    }
}

/// Writes to a temporary file first so a crash never leaves half a file.
fn save_book(path: &Path, book: &OpponentBook) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(book).map_err(io::Error::other)?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, json)?;
    fs::rename(&temporary, path)
}

/// Reads one frame of the table and returns the advice for the hero, the
/// hand history and the opponents' profiles. Saves the opponent stats when
/// the frame starts a new hand.
#[tauri::command]
pub async fn observe_table(
    snapshot: TableSnapshot,
    state: State<'_, AdvisorState>,
) -> Result<TableUpdate, String> {
    let mut session = state.session()?;
    let update = session
        .observe(snapshot)
        .map_err(|error| error.to_string())?;
    if update.recorded_a_finished_hand {
        state.save(session.book())?;
    }
    Ok(update)
}

#[tauri::command]
pub async fn opponent_population(state: State<'_, AdvisorState>) -> Result<Population, String> {
    Ok(state.session()?.book().population())
}

/// Sets the population that new and lightly observed players are assumed
/// to belong to, and saves it.
#[tauri::command]
pub async fn set_opponent_population(
    population: Population,
    state: State<'_, AdvisorState>,
) -> Result<(), String> {
    let mut session = state.session()?;
    session.book_mut().set_population(population);
    state.save(session.book())
}

/// Forgets every opponent's stats, keeping the chosen population.
#[tauri::command]
pub async fn reset_opponents(state: State<'_, AdvisorState>) -> Result<(), String> {
    let mut session = state.session()?;
    let population = session.book().population();
    *session.book_mut() = OpponentBook::new(population);
    state.save(session.book())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("poker-helper-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn saves_and_loads_the_book() {
        let dir = scratch_dir("round-trip");
        let path = dir.join(BOOK_FILE_NAME);
        let book = OpponentBook::new(Population::Regular);
        save_book(&path, &book).unwrap();
        assert_eq!(load_book_or_set_aside(&path), book);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn moves_an_unreadable_file_aside() {
        let dir = scratch_dir("unreadable");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(BOOK_FILE_NAME);
        fs::write(&path, "not json").unwrap();
        assert_eq!(load_book_or_set_aside(&path), OpponentBook::default());
        assert!(!path.exists());
        assert!(dir.join("opponents.json.unreadable").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
