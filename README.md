# DMARC Analyzer

A DMARC analyzer written in rust.

It will fetch the reports directly from an IMAP accounts INBOX. The reports are extracted, parsed and stored in a local SQLite database.
The results are shown in a simple web based interface.

For now, this is still very much in progress an there is not too much to see besides some basic stats and a listing of all the reports.


I tested the parser against over 500 reports that I had at hand. Mostly from Google and Yahoo. A few from other services as well.
If it fails parsing a report, I would be happy if you could make it available to me so I can adapt the parser.

Still TODO:
- [ ] move mails to other folder once parsed and stored

## Dependencies

- SQLite 3.6.8 or newer
- OpenSSL

## Installation

1. Clone this repository
2. Set it to use rust nightly (needed because of Rocket): `rustup override set nightly`
3. Adapt the `config.cfg` file to point to you IMAP account that has the DMARC reports.
4. run `cargo run`
5. Fetch reports either via the GUI or by running `curl http://localhost:8000/fetch`

## Changelog:

### 0.2.0
Reworked around Rocket to provide a web interface.

### 0.1.0
Fetch, parse and store reports
