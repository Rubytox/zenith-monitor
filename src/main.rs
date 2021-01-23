mod db;
mod grades;
mod mail;

use rusqlite::Result;
use serde_json::Value;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

struct Profile {
    zenith_username: String,
    zenith_password: String,
    mailer_username: String,
    mailer_password: String,
    mailer_server: String,
    mailer_from: String,
    mailer_to: String,
}

fn read_profile() -> Result<Profile> {
    let json = fs::read_to_string("profile.json").unwrap();
    let json: Value = serde_json::from_str(&json).unwrap();

    Ok(Profile {
        zenith_username: json["zenith"]["username"].as_str().unwrap().to_owned(),
        zenith_password: json["zenith"]["password"].as_str().unwrap().to_owned(),
        mailer_username: json["mailer"]["username"].as_str().unwrap().to_owned(),
        mailer_password: json["mailer"]["password"].as_str().unwrap().to_owned(),
        mailer_server: json["mailer"]["server"].as_str().unwrap().to_owned(),
        mailer_from: json["mailer"]["from"].as_str().unwrap().to_owned(),
        mailer_to: json["mailer"]["to"].as_str().unwrap().to_owned(),
    })
}

fn main() {
    ctrlc::set_handler(move || {
        println!("Exiting cleanly");
        std::process::exit(0);
    })
    .unwrap();

    let conn = db::init_db().unwrap();
    println!("Connected to the database");

    loop {
        let profile = read_profile().unwrap();
        println!("Read profile correctly");

        let fetched_grades =
            grades::fetch_grades(profile.zenith_username, profile.zenith_password).unwrap();
        println!("Fetched grades: {:?}", fetched_grades);
        let stored_grades = db::get_grades(&conn).unwrap();
        println!("Stored grades: {:?}", stored_grades);

        let (new, updated) = grades::split_grades(fetched_grades, stored_grades).unwrap();
        println!("New grades: {:?}", new);
        println!("Updated grades: {:?}", updated);

        if !new.is_empty() || !updated.is_empty() {
            println!("There are new or updated grades!");
            let mailer = mail::init_mailer(
                profile.mailer_username.as_str(),
                profile.mailer_password.as_str(),
                profile.mailer_server.as_str(),
                profile.mailer_from.as_str(),
                profile.mailer_to.as_str(),
            )
            .unwrap();
            println!("Created mailer");

            mail::notify_grade(&mailer, &new, &updated);
            println!("E-mail was sent");

            new.iter().for_each(|(s, g)| {
                db::insert_grade(&conn, s, g).unwrap();
            });

            updated.iter().for_each(|(s, (_, g))| {
                db::update_grade(&conn, s, g).unwrap();
            });
            println!("Updated database");
        }

        // Wait an hour
        println!("Pause for one hour");
        sleep(Duration::from_secs(3600));
    }
}
