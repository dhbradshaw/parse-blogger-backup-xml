# Parse Blogger Backup

I had the need to parse a series of backup xml files from Blogger into Rust to manipulate further and I wrote some rough code to do it.

So that the code doesn't get lost, I'm open sourcing it here.

Basically, it uses quick-xml to walk through the xml and turns that event stream into a list of entries of different types.

It then connects comments to their post parents and returns the data.

It makes no effort to parse the html stored, but just adds it as a string to the Post or Comment instance.

## Usage

Provide the `get_posts` function with a path string showing it where to look for your backup file.

It will return post objects that you can then manipulate.
