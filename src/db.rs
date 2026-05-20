use rusqlite::{Connection, Result, params};
use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Installation {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub active: bool,
    pub installed_at: Option<String>,
}

fn ensure_data_dir() -> std::path::PathBuf {
    let dir = dirs_next::data_dir()
        .expect("Could not determine data directory")
        .join("pif");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Could not create pif data directory");
    }
    dir
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS installations (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT NOT NULL,
            path         TEXT NOT NULL UNIQUE,
            version      TEXT,
            active       BOOLEAN NOT NULL DEFAULT 1,
            installed_at TEXT
        );"
    )?;

    // Migrate existing databases that predate the version column.
    let _ = conn.execute(
        "ALTER TABLE installations ADD COLUMN version TEXT", []
    );

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schema_version", [], |r| r.get(0)
    )?;
    if count == 0 {
        conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
    }

    Ok(())
}

pub fn get_or_create_table() -> Result<Connection> {
    let db_path = ensure_data_dir().join("stat.db");
    let conn = Connection::open(&db_path)?;
    create_tables(&conn)?;
    Ok(conn)
}

pub fn is_installed(conn: &Connection, name: &str, path: &str, version: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM installations WHERE name = ?1 AND path = ?2 AND version = ?3",
        params![name, path, version],
        |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0
}

pub fn record_installation(conn: &Connection, name: &str, installation_path: &str, version: &str) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    match conn.execute(
        "INSERT INTO installations (name, path, version, active, installed_at) VALUES (?1, ?2, ?3, 1, ?4)
         ON CONFLICT(path) DO UPDATE SET name=excluded.name, version=excluded.version, active=1, installed_at=excluded.installed_at",
        params![name, installation_path, version, now]
    ) {
        Ok(_) => {}
        Err(e) => eprintln!("Error recording installation: {:?}", e),
    }
}

pub fn list_installations(conn: &Connection) -> Result<Vec<Installation>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, version, active, installed_at FROM installations ORDER BY name, path"
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Installation {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                version: row.get(3)?,
                active: row.get(4)?,
                installed_at: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

pub fn get_installations_by_name(conn: &Connection, name: &str) -> Result<Vec<Installation>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, version, active, installed_at FROM installations WHERE name = ?1 ORDER BY path"
    )?;
    let rows = stmt
        .query_map(params![name], |row| {
            Ok(Installation {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                version: row.get(3)?,
                active: row.get(4)?,
                installed_at: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

pub fn remove_installation(conn: &Connection, name: &str, path_filter: Option<&str>) -> Result<usize> {
    match path_filter {
        Some(path) => conn.execute(
            "DELETE FROM installations WHERE name = ?1 AND path = ?2",
            params![name, path]
        ),
        None => conn.execute(
            "DELETE FROM installations WHERE name = ?1",
            params![name]
        ),
    }
}

pub fn clean_stale_installations(conn: &Connection) -> Result<usize> {
    let installations = list_installations(conn)?;
    let stale: Vec<i32> = installations
        .iter()
        .filter(|i| !Path::new(&i.path).exists())
        .map(|i| i.id)
        .collect();
    let removed = stale.len();
    for id in stale {
        conn.execute("DELETE FROM installations WHERE id = ?1", params![id])?;
    }
    Ok(removed)
}

pub fn print_installations(conn: &Connection) -> Result<()> {
    let installations = list_installations(conn)?;
    if installations.is_empty() {
        println!("No installations recorded in the registry.");
        return Ok(());
    }

    let name_w = installations.iter().map(|i| i.name.len()).max().unwrap_or(4).max(4);
    let ver_w  = installations.iter()
        .map(|i| i.version.as_deref().unwrap_or("-").len())
        .max().unwrap_or(7).max(7);

    println!("{:<name_w$}  {:<ver_w$}  {}", "Name", "Version", "Path", name_w = name_w, ver_w = ver_w);
    println!("{}", "-".repeat(name_w + ver_w + 2 + 4 + 40));
    for i in &installations {
        let marker = if Path::new(&i.path).exists() { "" } else { "  [missing]" };
        let ver = i.version.as_deref().unwrap_or("-");
        println!("{:<name_w$}  {:<ver_w$}  {}{}", i.name, ver, i.path, marker, name_w = name_w, ver_w = ver_w);
    }
    Ok(())
}
