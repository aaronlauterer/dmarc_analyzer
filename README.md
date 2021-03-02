# DMARC Analyzer

A DMARC analyzer written in rust.

For now, this is still very much in progress an there is not too much to see.

What works, is to connect to an IMAP account, fetch the reports and store them in an sqlite database once parsed.

I tested the parser against over 500 reports that I had at hand. Mostly from Google and Yahoo. A few from other services as well.

Still TODO:
- [ ] move mails to other folder once parsed and stored
- [ ] create interface/api to get useful data out of it
