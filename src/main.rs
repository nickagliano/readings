use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use chrono_tz::America::New_York;
use serde::Deserialize;
use serde_json::{json, Value};

// Universalis JSONP API — returns the day's Mass readings.
// Docs: https://universalis.com/n-jsonp-technical.htm

#[derive(Deserialize)]
struct Reading {
    source:  String,
    text:    String,
    heading: Option<String>,
}

#[derive(Deserialize)]
struct DayReadings {
    day:   String,
    #[serde(rename = "Mass_R1")] r1:    Reading,
    #[serde(rename = "Mass_R2")] r2:    Option<Reading>,
    #[serde(rename = "Mass_Ps")] psalm: Option<Reading>,
    #[serde(rename = "Mass_G")]  gospel: Reading,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(5545);

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health));

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await.unwrap();
    println!("[readings] listening on {host}:{port}");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn index() -> Response {
    match fetch_readings().await {
        Ok(data) => html_response(StatusCode::OK, render_html(&data)),
        Err(e) => {
            eprintln!("[readings] fetch error: {e}");
            html_response(StatusCode::INTERNAL_SERVER_ERROR, error_html(&e.to_string()))
        }
    }
}

async fn fetch_readings() -> Result<DayReadings, Box<dyn std::error::Error>> {
    let date = Utc::now().with_timezone(&New_York).format("%Y%m%d").to_string();
    let url  = format!("https://universalis.com/0/{date}/jsonpmass.js?callback=x");

    let raw = reqwest::get(&url).await?.text().await?;

    let json_str = raw
        .trim()
        .strip_prefix("x(")
        .and_then(|s| s.strip_suffix(");"))
        .ok_or("unexpected JSONP format")?;

    Ok(serde_json::from_str(json_str)?)
}

fn html_response(status: StatusCode, body: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html; charset=utf-8".parse().unwrap());
    (status, headers, body).into_response()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn clean_text(s: &str) -> String {
    // Convert <br> variants to newlines, then strip remaining HTML tags.
    let s = s.replace("<br />", "\n").replace("<br/>", "\n").replace("<br>", "\n");
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<'           => in_tag = true,
            '>'           => in_tag = false,
            c if !in_tag  => out.push(c),
            _             => {}
        }
    }
    out
}

fn render_section(label: &str, r: &Reading) -> String {
    let label  = html_escape(label);
    let source = clean_text(&r.source); // preserve HTML entities (e.g. &#x2010; en-dash)
    // text is rendered as raw HTML — Universalis provides browser-ready markup
    // (prose readings use a single justify-aligned div; psalm uses per-line divs with hanging indent)
    let text = &r.text;
    format!(r#"  <section>
    <h2>{label}</h2>
    <div class="citation">{source}</div>
    <div class="reading-text">{text}</div>
  </section>
"#)
}

fn render_html(data: &DayReadings) -> String {
    let day  = html_escape(&clean_text(&data.day));
    let date = Utc::now().with_timezone(&New_York).format("%A, %B %-d, %Y").to_string();

    let r1_label = data.r1.heading.as_deref().unwrap_or("First Reading");
    let mut sections = render_section(r1_label, &data.r1);

    if let Some(ps) = &data.psalm {
        let label = ps.heading.as_deref().unwrap_or("Responsorial Psalm");
        sections.push_str(&render_section(label, ps));
    }
    if let Some(r2) = &data.r2 {
        let label = r2.heading.as_deref().unwrap_or("Second Reading");
        sections.push_str(&render_section(label, r2));
    }

    let gospel_label = data.gospel.heading.as_deref().unwrap_or("Gospel");
    sections.push_str(&render_section(gospel_label, &data.gospel));

    let css = r#"<style>
    * { box-sizing: border-box; }
    body {
      font-family: Georgia, 'Times New Roman', serif;
      font-size: 18px;
      line-height: 1.7;
      color: #1a1a1a;
      background: #fff;
      max-width: 680px;
      margin: 0 auto;
      padding: 2rem 1.5rem 4rem;
    }
    header {
      border-bottom: 1px solid #ddd;
      padding-bottom: 1rem;
      margin-bottom: 2.5rem;
    }
    h1 {
      font-size: 1.5rem;
      margin: 0 0 0.3rem;
      font-weight: normal;
    }
    .date {
      color: #888;
      font-size: 0.9rem;
      font-style: italic;
    }
    section {
      margin-bottom: 2.5rem;
    }
    h2 {
      font-size: 0.8rem;
      text-transform: uppercase;
      letter-spacing: 0.1em;
      color: #999;
      margin: 0 0 0.2rem;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    }
    .citation {
      color: #555;
      font-size: 0.95rem;
      margin-bottom: 0.8rem;
      font-style: italic;
    }
    .reading-text {
      margin: 0;
    }
    .reading-text div {
      margin-bottom: 0;
      text-align: left !important;
    }
  </style>"#;

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{day}</title>
  {css}
</head>
<body>
  <header>
    <h1>{day}</h1>
    <div class="date">{date}</div>
  </header>
{sections}
</body>
</html>"#)
}

fn error_html(msg: &str) -> String {
    let msg = html_escape(msg);
    let css = r#"<style>
    body { font-family: Georgia, serif; max-width: 680px; margin: 4rem auto; padding: 0 1.5rem; color: #1a1a1a; }
    h1 { font-size: 1.2rem; font-weight: normal; }
    p { color: #666; font-size: 0.95rem; }
  </style>"#;
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Error — Readings</title>
  {css}
</head>
<body>
  <h1>Couldn't load today's readings</h1>
  <p>{msg}</p>
</body>
</html>"#)
}
