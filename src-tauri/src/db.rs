use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub type Db = Mutex<Connection>;

pub fn init(conn: Connection) -> rusqlite::Result<Db> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY, content TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT 'yellow',
            x REAL NOT NULL DEFAULT 120, y REAL NOT NULL DEFAULT 40,
            w REAL NOT NULL DEFAULT 240, h REAL NOT NULL DEFAULT 170,
            snooze_minutes INTEGER NOT NULL DEFAULT 2,
            created_at TEXT NOT NULL,
            completed_at TEXT, is_hidden INTEGER NOT NULL DEFAULT 0, hidden_until TEXT
        );
        CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, val TEXT NOT NULL);",
    )?;
    Ok(Mutex::new(conn))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub color: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub snooze_minutes: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub is_hidden: bool,
    pub hidden_until: Option<String>,
}

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        content: row.get(1)?,
        color: row.get(2)?,
        x: row.get(3)?,
        y: row.get(4)?,
        w: row.get(5)?,
        h: row.get(6)?,
        snooze_minutes: row.get(7)?,
        created_at: row.get(8)?,
        completed_at: row.get(9)?,
        is_hidden: row.get::<_, i64>(10)? != 0,
        hidden_until: row.get(11)?,
    })
}

pub struct NoteRepository;

impl NoteRepository {
    pub fn active(db: &Db) -> Result<Vec<Note>, String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        let mut stmt = lock
            .prepare("SELECT * FROM notes WHERE completed_at IS NULL ORDER BY created_at")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], row_to_note).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn completed(db: &Db) -> Result<Vec<Note>, String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        let mut stmt = lock
            .prepare("SELECT * FROM notes WHERE completed_at IS NOT NULL ORDER BY completed_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], row_to_note).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn get(db: &Db, id: &str) -> Result<Option<Note>, String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.query_row("SELECT * FROM notes WHERE id = ?1", params![id], row_to_note)
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn create(db: &Db, n: &Note) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute(
            "INSERT INTO notes (id, content, color, x, y, w, h, snooze_minutes, created_at, completed_at, is_hidden, hidden_until)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![n.id, n.content, n.color, n.x, n.y, n.w, n.h, n.snooze_minutes,
                    n.created_at, n.completed_at, n.is_hidden as i64, n.hidden_until],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_position(db: &Db, id: &str, x: f64, y: f64) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET x=?1, y=?2 WHERE id=?3", params![x, y, id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_content(db: &Db, id: &str, content: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET content=?1 WHERE id=?2", params![content, id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn snooze(db: &Db, id: &str, until_iso: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET is_hidden=1, hidden_until=?1 WHERE id=?2", params![until_iso, id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_snooze(db: &Db, id: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET is_hidden=0, hidden_until=NULL WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn complete(db: &Db, id: &str, at_iso: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET completed_at=?1, is_hidden=0, hidden_until=NULL WHERE id=?2", params![at_iso, id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reactivate(db: &Db, id: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("UPDATE notes SET completed_at=NULL, is_hidden=0, hidden_until=NULL WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete(db: &Db, id: &str) -> Result<(), String> {
        let lock = db.lock().map_err(|e| e.to_string())?;
        lock.execute("DELETE FROM notes WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mem() -> Db { init(Connection::open_in_memory().unwrap()).unwrap() }
    fn sample(id: &str) -> Note {
        Note { id: id.into(), content: "c".into(), color: "yellow".into(), x: 0.0, y: 0.0,
               w: 240.0, h: 170.0, snooze_minutes: 2, created_at: "2026-07-22T10:00:00Z".into(),
               completed_at: None, is_hidden: false, hidden_until: None }
    }

    #[test]
    fn active_completed_partition() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        NoteRepository::complete(&db, "a", "2026-07-22T11:00:00Z").unwrap();
        NoteRepository::create(&db, &sample("b")).unwrap();
        assert_eq!(NoteRepository::active(&db).unwrap().len(), 1);
        assert_eq!(NoteRepository::completed(&db).unwrap().len(), 1);
    }

    #[test]
    fn snooze_sets_then_clears() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        NoteRepository::snooze(&db, "a", "2026-07-22T10:05:00Z").unwrap();
        assert!(NoteRepository::get(&db, "a").unwrap().unwrap().is_hidden);
        NoteRepository::clear_snooze(&db, "a").unwrap();
        assert!(!NoteRepository::get(&db, "a").unwrap().unwrap().is_hidden);
    }

    #[test]
    fn reactivate_brings_back() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        NoteRepository::complete(&db, "a", "2026-07-22T11:00:00Z").unwrap();
        assert!(NoteRepository::active(&db).unwrap().is_empty());
        NoteRepository::reactivate(&db, "a").unwrap();
        assert_eq!(NoteRepository::active(&db).unwrap().len(), 1);
    }
}
