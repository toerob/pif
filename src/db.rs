use rusqlite::{ Connection, Result, params };

fn create_table(conn: &Connection) -> Result<()> {
    if
        let Err(e) = conn.execute(
            "CREATE TABLE IF NOT EXISTS installations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNQIUE NOT NULL,
                path TEXT UNIQUE NOT NULL,
                active BOOLEAN
            )",
            []
        )
    {
        //print_warning_msg(true, "Tabellen kunde ej skapas");
        eprintln!("Tabellen kunde ej skapas: {}", e);
    }
    println!("Tabellen skapades (om den inte redan fanns).");
    Ok(())
}

pub fn record_installation(conn: &Connection, name: &str, installation_path: &str) {
    match conn.execute("INSERT INTO installations (name, path, active) VALUES (?1, ?2, true)",
        params![name, installation_path]
    ) {
        Ok(msg) => {
            println!("Installation added: {} ({})", name, installation_path);
        },
        Err(e) => {
            match &e.sqlite_error_code() {
                Some(SQLITE_CONSTRAINT_UNIQUE) => {
                    /*
                    fine with this
                    */
                    return;
                },
                None => {
                },
            }
            eprintln!("Error happened while adding record to table: {:?}", e);
        }
    }
    //Ok(())
}


#[derive(Debug)]
struct Installation {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub active: bool,
}

pub fn check_installations(conn: &Connection) -> Result<()> {
    println!("Installations made...");

    //let mut query = db.prepare("select * from installations")?;
    //let result = conn.query_row("select name,path from installations", [], |row| row.get(0));

    let mut stmt = conn.prepare("select id,name,path,active from installations;")?;


    let installation_iter = stmt.query_map([], |row| {
        Ok(Installation {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            active: row.get(3)?,
        })
    }).unwrap()
    .filter_map(|x|x.ok())
    .filter(|x| std::path::Path::new(&x.path).exists());
    
    // TODO:
    //let broken_installations = installation_iter.cloned()
    //    .filter(|x| std::path::Path::new(&x.path).exists());


    println!("Found installations:"); 
    let _ = &for installation in installation_iter {
        println!("{}: {} ", installation.name, installation.path);
    };
    /*
    let rows = query.query_map([], |row| {
        let x = row.get_ref(0)?.as_str()?; // check From<FromSqlError> for Error
        Ok(x[..].to_owned())
    })?;
    */
    Ok(())
}

pub fn get_or_create_table() -> Result<Connection> {
    let file_path = dirs_next
        ::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("stat.db")
        .clone();

    let db_path = file_path.as_path().to_str().clone().expect("Conversion path->str failed");
    let conn = Connection::open(db_path).expect("Could not open/create db");

    println!("Ansluten till databasen!");
    create_table(&conn)?;
    Ok((conn))
}
