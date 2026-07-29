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

/// 统一错误转换:任何 ToString 错误 → String(前端契约)。供 commands/autostart 复用。
pub(crate) fn to_str<E: ToString>(e: E) -> String {
    e.to_string()
}

/// 取锁 → 在 Connection 上执行 rusqlite 闭包 → 统一转 String。
/// 收敛每个 repository 方法的「取锁 map_err + execute map_err」三步样板。
fn run<F, R>(db: &Db, f: F) -> Result<R, String>
where
    F: FnOnce(&Connection) -> rusqlite::Result<R>,
{
    let conn = db.lock().map_err(to_str)?;
    f(&conn).map_err(to_str)
}

/// execute 专用:丢弃受影响行数(usize)→()。INSERT/UPDATE/DELETE 用它。
fn run_exec<F>(db: &Db, f: F) -> Result<(), String>
where
    F: FnOnce(&Connection) -> rusqlite::Result<usize>,
{
    run(db, |c| f(c).map(|_| ()))
}

/// 显式列名(顺序对齐 row_to_note 的 0–11),替代脆弱的 SELECT *。
const NOTES_COLS: &str = "id, content, color, x, y, w, h, snooze_minutes, created_at, completed_at, is_hidden, hidden_until";

pub struct NoteRepository;

impl NoteRepository {
    pub fn active(db: &Db) -> Result<Vec<Note>, String> {
        run(db, |c| {
            let mut stmt = c.prepare(&format!("SELECT {NOTES_COLS} FROM notes WHERE completed_at IS NULL ORDER BY created_at"))?;
            let rows = stmt.query_map([], row_to_note)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn completed(db: &Db) -> Result<Vec<Note>, String> {
        run(db, |c| {
            let mut stmt = c.prepare(&format!("SELECT {NOTES_COLS} FROM notes WHERE completed_at IS NOT NULL ORDER BY completed_at DESC"))?;
            let rows = stmt.query_map([], row_to_note)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn get(db: &Db, id: &str) -> Result<Option<Note>, String> {
        run(db, |c| {
            c.query_row(&format!("SELECT {NOTES_COLS} FROM notes WHERE id = ?1"), params![id], row_to_note).optional()
        })
    }

    pub fn create(db: &Db, n: &Note) -> Result<(), String> {
        run_exec(db, |c| c.execute(
            "INSERT INTO notes (id, content, color, x, y, w, h, snooze_minutes, created_at, completed_at, is_hidden, hidden_until)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![n.id, n.content, n.color, n.x, n.y, n.w, n.h, n.snooze_minutes,
                    n.created_at, n.completed_at, n.is_hidden as i64, n.hidden_until],
        ))
    }

    pub fn update_position(db: &Db, id: &str, x: f64, y: f64) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET x=?1, y=?2 WHERE id=?3", params![x, y, id]))
    }

    pub fn update_content(db: &Db, id: &str, content: &str) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET content=?1 WHERE id=?2", params![content, id]))
    }

    pub fn update_color(db: &Db, id: &str, color: &str) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET color=?1 WHERE id=?2", params![color, id]))
    }

    pub fn update_size(db: &Db, id: &str, w: f64, h: f64) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET w=?1, h=?2 WHERE id=?3", params![w, h, id]))
    }

    pub fn update_snooze_minutes(db: &Db, id: &str, mins: i64) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET snooze_minutes=?1 WHERE id=?2", params![mins, id]))
    }

    pub fn snooze(db: &Db, id: &str, until_iso: &str) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET is_hidden=1, hidden_until=?1 WHERE id=?2", params![until_iso, id]))
    }

    pub fn clear_snooze(db: &Db, id: &str) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET is_hidden=0, hidden_until=NULL WHERE id=?1", params![id]))
    }

    pub fn complete(db: &Db, id: &str, at_iso: &str) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET completed_at=?1, is_hidden=0, hidden_until=NULL WHERE id=?2", params![at_iso, id]))
    }

    pub fn reactivate(db: &Db, id: &str) -> Result<(), String> {
        run_exec(db, |c| c.execute("UPDATE notes SET completed_at=NULL, is_hidden=0, hidden_until=NULL WHERE id=?1", params![id]))
    }

    pub fn delete(db: &Db, id: &str) -> Result<(), String> {
        run_exec(db, |c| c.execute("DELETE FROM notes WHERE id=?1", params![id]))
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

    #[test]
    fn update_color_changes_color() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        assert_eq!(NoteRepository::get(&db, "a").unwrap().unwrap().color, "yellow");
        NoteRepository::update_color(&db, "a", "pink").unwrap();
        assert_eq!(NoteRepository::get(&db, "a").unwrap().unwrap().color, "pink");
    }

    #[test]
    fn update_size_changes_dimensions() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        let before = NoteRepository::get(&db, "a").unwrap().unwrap();
        assert_eq!((before.w, before.h), (240.0, 170.0));
        NoteRepository::update_size(&db, "a", 360.0, 260.0).unwrap();
        let after = NoteRepository::get(&db, "a").unwrap().unwrap();
        assert_eq!((after.w, after.h), (360.0, 260.0));
    }

    #[test]
    fn update_snooze_minutes_changes_duration() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        assert_eq!(NoteRepository::get(&db, "a").unwrap().unwrap().snooze_minutes, 2);
        NoteRepository::update_snooze_minutes(&db, "a", 10).unwrap();
        assert_eq!(NoteRepository::get(&db, "a").unwrap().unwrap().snooze_minutes, 10);
    }

    #[test]
    fn row_roundtrip_preserves_all_fields() {
        let db = mem();
        let original = Note {
            id: "x".into(), content: "内容".into(), color: "blue".into(),
            x: 12.5, y: 34.5, w: 360.0, h: 260.0, snooze_minutes: 10,
            created_at: "2026-07-22T10:00:00Z".into(),
            completed_at: Some("2026-07-22T11:00:00Z".into()),
            is_hidden: true, // i64↔bool 转换点 (CLAUDE.md 强调)
            hidden_until: Some("2026-07-22T10:05:00Z".into()),
        };
        NoteRepository::create(&db, &original).unwrap();
        assert_eq!(NoteRepository::get(&db, "x").unwrap().unwrap(), original);
    }

    #[test]
    fn update_position_and_content_roundtrip() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        NoteRepository::update_position(&db, "a", 200.0, 300.0).unwrap();
        let n = NoteRepository::get(&db, "a").unwrap().unwrap();
        assert_eq!((n.x, n.y), (200.0, 300.0));
        NoteRepository::update_content(&db, "a", "新文本").unwrap();
        assert_eq!(NoteRepository::get(&db, "a").unwrap().unwrap().content, "新文本");
    }

    #[test]
    fn delete_removes_row() {
        let db = mem();
        NoteRepository::create(&db, &sample("a")).unwrap();
        NoteRepository::delete(&db, "a").unwrap();
        assert!(NoteRepository::get(&db, "a").unwrap().is_none());
    }

    #[test]
    fn get_missing_returns_none() {
        let db = mem();
        assert!(NoteRepository::get(&db, "nope").unwrap().is_none());
    }
}
