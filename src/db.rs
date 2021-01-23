use rusqlite::{params, Connection, OpenFlags, Result};
use std::collections::HashMap;

struct Grade {
    subject: String,
    value: f64,
}

pub fn create_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE grades (
            id          INTEGER PRIMARY KEY,
            subject     TEXT NOT NULL,
            grade       REAL
        )",
        params![],
    )?;

    Ok(())
}

pub fn init_db() -> Result<Connection> {
    match Connection::open_with_flags("grades.db", OpenFlags::SQLITE_OPEN_READ_WRITE) {
        Ok(conn) => Ok(conn),
        Err(_) => {
            let conn = Connection::open("grades.db")?;
            create_db(&conn)?;
            Ok(conn)
        }
    }
}

pub fn insert_grade(conn: &Connection, subject: &String, grade: &f64) -> Result<usize> {
    conn.execute(
        "INSERT INTO grades (subject, grade)
             VALUES (?1, ?2)",
        params![subject, grade],
    )
}

pub fn update_grade(conn: &Connection, subject: &String, grade: &f64) -> Result<usize> {
    conn.execute(
        "UPDATE grades SET grade = ?1 WHERE subject == ?2",
        params![grade, subject],
    )
}

pub fn get_grades(conn: &Connection) -> Result<HashMap<String, f64>> {
    let mut grades: HashMap<String, f64> = HashMap::new();

    let mut stmt = conn.prepare("SELECT subject, grade FROM grades")?;
    let grades_iter = stmt.query_map(params![], |row| {
        Ok(Grade {
            subject: row.get(0)?,
            value: row.get(1)?,
        })
    })?;

    grades.extend(grades_iter.map(|g| {
        let grade = g.unwrap();
        (grade.subject, grade.value)
    }));

    Ok(grades)
}
