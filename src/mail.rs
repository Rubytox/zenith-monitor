use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rusqlite::Result;
use std::collections::HashMap;

pub struct Mailer {
    mailer: SmtpTransport,
    from: String,
    to: String,
}

pub fn init_mailer(
    username: &str,
    password: &str,
    server: &str,
    from: &str,
    to: &str,
) -> Result<Mailer> {
    let creds = Credentials::new(username.to_string(), password.to_string());
    let mailer = SmtpTransport::relay(server)
        .unwrap()
        .credentials(creds)
        .build();

    Ok(Mailer {
        mailer,
        from: from.to_owned(),
        to: to.to_owned(),
    })
}

pub fn notify_grade(
    mailer: &Mailer,
    new_grades: &HashMap<String, f64>,
    updated_grades: &HashMap<String, (f64, f64)>,
) {
    let mut body = "".to_owned();
    new_grades.iter().for_each(|(s, g)| {
        body.push_str(&*format!("[New] {}: {}\n", s, g));
    });
    updated_grades.iter().for_each(|(s, (old_g, new_g))| {
        body.push_str(&*format!("[Updated] {}: {} -> {}\n", s, old_g, new_g));
    });

    let email = Message::builder()
        .from(mailer.from.parse().unwrap())
        .to(mailer.to.parse().unwrap())
        //.from("Zenith Grades <rubytox@rubytox.fr>".parse().unwrap())
        //.to("Christophe Néraud <christophe.neraud@grenoble-inp.org>"
        //    .parse()
        //    .unwrap())
        .subject("New grade!")
        .body(body)
        .unwrap();

    match mailer.mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
