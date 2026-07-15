use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub struct IndexDb {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub size: u64,
    pub mtime: u64,
    pub hash: String,
    pub language: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SymbolRecord {
    pub id: i64,
    pub file_id: i64,
    pub name: String,
    pub kind: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl IndexDb {
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_default();
        }

        let conn = Connection::open(db_path)?;

        // Enable WAL mode, Foreign Keys and normal synchronous writing for maximum safety & speed
        conn.execute("PRAGMA journal_mode = WAL;", [])?;
        conn.execute("PRAGMA foreign_keys = ON;", [])?;
        conn.execute("PRAGMA synchronous = NORMAL;", [])?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                size INTEGER NOT NULL,
                mtime INTEGER NOT NULL,
                hash TEXT NOT NULL,
                language TEXT NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                start_column INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                end_column INTEGER NOT NULL,
                FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS dependencies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_file_id INTEGER NOT NULL,
                target_file_id INTEGER NOT NULL,
                kind TEXT NOT NULL,
                FOREIGN KEY(source_file_id) REFERENCES files(id) ON DELETE CASCADE,
                FOREIGN KEY(target_file_id) REFERENCES files(id) ON DELETE CASCADE
            );",
            [],
        )?;

        // Create performance-critical indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_symbols_file_id ON symbols(file_id);",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dependencies_source ON dependencies(source_file_id);",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dependencies_target ON dependencies(target_file_id);",
            [],
        )?;

        Ok(())
    }

    pub fn insert_file(
        &self,
        path: &str,
        size: u64,
        mtime: u64,
        hash: &str,
        language: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO files (path, size, mtime, hash, language)
             VALUES (?1, ?2, ?3, ?4, ?5);",
            params![path, size as i64, mtime as i64, hash, language],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_file(&self, path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Because of ON DELETE CASCADE, deleting a file record automatically deletes its symbols!
        conn.execute("DELETE FROM files WHERE path = ?1;", [path])?;
        Ok(())
    }

    pub fn get_file(&self, path: &str) -> Result<Option<FileRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, path, size, mtime, hash, language FROM files WHERE path = ?1;")?;
        let mut rows = stmt.query([path])?;

        if let Some(row) = rows.next()? {
            let size_int: i64 = row.get(2)?;
            let mtime_int: i64 = row.get(3)?;
            Ok(Some(FileRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                size: size_int as u64,
                mtime: mtime_int as u64,
                hash: row.get(4)?,
                language: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert_symbol(&self, sym: &SymbolRecord) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO symbols (file_id, name, kind, start_line, start_column, end_line, end_column)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            params![
                sym.file_id,
                sym.name,
                sym.kind,
                sym.start_line as i64,
                sym.start_column as i64,
                sym.end_line as i64,
                sym.end_column as i64
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_symbols_for_file(&self, file_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM symbols WHERE file_id = ?1;", params![file_id])?;
        Ok(())
    }

    pub fn get_symbols_for_file(&self, file_id: i64) -> Result<Vec<SymbolRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, name, kind, start_line, start_column, end_line, end_column
             FROM symbols WHERE file_id = ?1;",
        )?;
        let mut rows = stmt.query(params![file_id])?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let sl: i64 = row.get(4)?;
            let sc: i64 = row.get(5)?;
            let el: i64 = row.get(6)?;
            let ec: i64 = row.get(7)?;
            list.push(SymbolRecord {
                id: row.get(0)?,
                file_id: row.get(1)?,
                name: row.get(2)?,
                kind: row.get(3)?,
                start_line: sl as usize,
                start_column: sc as usize,
                end_line: el as usize,
                end_column: ec as usize,
            });
        }
        Ok(list)
    }

    pub fn list_files(&self) -> Result<Vec<FileRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, path, size, mtime, hash, language FROM files;")?;
        let mut rows = stmt.query([])?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let size_int: i64 = row.get(2)?;
            let mtime_int: i64 = row.get(3)?;
            list.push(FileRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                size: size_int as u64,
                mtime: mtime_int as u64,
                hash: row.get(4)?,
                language: row.get(5)?,
            });
        }
        Ok(list)
    }

    pub fn insert_dependency(
        &self,
        source_file_id: i64,
        target_file_id: i64,
        kind: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO dependencies (source_file_id, target_file_id, kind)
             VALUES (?1, ?2, ?3);",
            params![source_file_id, target_file_id, kind],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_dependencies_for_file(&self, source_file_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM dependencies WHERE source_file_id = ?1;",
            params![source_file_id],
        )?;
        Ok(())
    }

    pub fn get_dependencies_for_file(&self, source_file_id: i64) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT target_file_id, kind FROM dependencies WHERE source_file_id = ?1;")?;
        let mut rows = stmt.query(params![source_file_id])?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let target_id: i64 = row.get(0)?;
            let kind: String = row.get(1)?;
            list.push((target_id, kind));
        }
        Ok(list)
    }
}
