# readings — Customization Guide

readings is a web reader for the Catholic daily Mass readings. Visit it in your browser
to read today's First Reading, Responsorial Psalm, and Gospel with proper liturgical
formatting. It fetches live from Universalis (universalis.com), Jerusalem Bible
translation, on every page load.

## Ports

### `MESSAGE_FORMAT`

**What it does:** Controls how each reading section is rendered on the page.
**Default:** Label (h2), citation (italic), full text as Universalis HTML.
**How to customize:** Edit `fn render_section(label, reading)` and `fn render_html(data)`
in `src/main.rs`.

The `DayReadings` struct gives you:
- `data.day`    — liturgical day name, e.g. "2nd Sunday of Lent"
- `data.r1`     — First Reading (always present)
- `data.r2`     — Second Reading (Sundays/feasts only, `None` on weekdays)
- `data.psalm`  — Responsorial Psalm (usually present)
- `data.gospel` — Gospel (always present)

Each reading has:
- `.source`  — citation, e.g. "Genesis 12:1‑4"
- `.text`    — full reading text as HTML (Universalis-formatted, safe to render directly)
- `.heading` — thematic title, e.g. "His face shone like the sun" (`Option<String>`)

## Caching

**Current behavior:** Fetches from Universalis on every page load. This is fine for a
single user — it's fast and always fresh.

**Future improvement:** The readings should be cached. Universalis publishes the full
liturgical calendar well in advance, so it would be straightforward to bulk-fetch an
entire year (or even multiple years) of readings and store them locally — probably just
a JSON file on disk keyed by date. A year's worth of readings is a small amount of data.
On startup (or on a daily cron), fetch-and-store any dates not yet cached; serve from
cache on page load. This would make the app work offline and eliminate the Universalis
dependency at read time.

## Getting Started

1. `chmod +x serve.sh && ./serve.sh`
2. Visit `http://localhost:5545`

Or via EPC: `epc deploy --local ./readings readings`

## Data source

Readings come from [Universalis](https://universalis.com), Jerusalem Bible translation.
The lectionary follows the Roman Rite calendar. URL pattern:
`https://universalis.com/0/YYYYMMDD/jsonpmass.js`
