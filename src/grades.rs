use rusqlite::Result;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub fn fetch_grades(username: String, password: String) -> reqwest::Result<HashMap<String, f64>> {
    let url = format!(
        "https://intranet.ensimag.fr/Zenith2/ConsultNotes?uid={}",
        username
    );
    let client = reqwest::blocking::Client::new();
    let html = client
        .get(&url)
        .basic_auth(username, Some(password))
        .send()?
        .text()?;

    let _code = r#"
    <!DOCTYPE html>
    <html>
        <head>
            <meta charset="UTF-8" />
            <title>Zenith grades</title>
        </head>
        <body>
            <table class="perso display">
                <thead>
                    <tr><th>Matiere</th><th>Année</th><th>Coef</th><th>Session</th><th>note</th><th>code absence</th></tr>
                </thead>
                <tbody>
                    <tr><td>SATRA</td><td>2020</td><td>3</td><td>1</td><td>12</td><td></td>
                    <tr><td>SECARCH</td><td>2020</td><td>6</td><td>1</td><td>16.2</td><td></td>
                </tbody>
            </table>
        </body>
    </html>
    "#;

    //let document = Html::parse_document(&code);
    let document = Html::parse_document(&html);

    let mut grades: HashMap<String, f64> = HashMap::new();

    let tbody_sel = Selector::parse("tbody").unwrap();
    let tr_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    let tbody = document.select(&tbody_sel).next().unwrap();
    // Now we descend in tbody and we look for tr, then six td
    // 0 : matiere
    // 1 : year
    // 2 : coef
    // 3 : session
    // 4 : grade
    // 5 : absence
    for row in tbody.select(&tr_sel) {
        let mut current_col = 0;
        let mut subject: Option<String> = None;
        let mut grade: Option<f64> = None;
        for col in row.select(&td_sel) {
            if current_col == 0 {
                let value = col.inner_html();
                subject = Some(value.into());
            }
            if current_col == 4 {
                grade = Some(col.inner_html()[..].parse().unwrap());
            }
            current_col += 1;
        }
        if let Some(s) = subject {
            if let Some(g) = grade {
                grades.insert(s, g);
            }
        }
    }

    Ok(grades)
}

pub fn split_grades(
    fetched: HashMap<String, f64>,
    stored: HashMap<String, f64>,
) -> Result<(HashMap<String, f64>, HashMap<String, (f64, f64)>)> {
    let mut new_grades: HashMap<String, f64> = HashMap::new();
    new_grades.extend(
        fetched
            .clone()
            .into_iter()
            .filter(|(s, _)| !stored.contains_key(s)),
    );
    let mut updated_grades: HashMap<String, (f64, f64)> = HashMap::new();
    updated_grades.extend(
        fetched
            .into_iter()
            .filter(|(s, g)| {
                stored.contains_key(s) && stored.get(s).unwrap().to_string() != g.to_string()
            })
            .map(|(s, g)| (s.clone(), (stored.get(&s).unwrap().to_owned(), g))),
    );

    Ok((new_grades, updated_grades))
}
