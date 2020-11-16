# game
There is a button that you can press every day to receive a point. Compete with your friends to see who can get the most points.

# what's the technical implementation?

* uses [Rocket](https://github.com/SergioBenitez/Rocket/) to serve webpages
* uses [tera](https://github.com/Keats/tera/) for templating
* uses a [sled](https://github.com/spacejam/sled/) database
* uses [bcrypt](https://github.com/Keats/rust-bcrypt) to hash/salt passwords
* pulls some trickery in order to embed static files and templates directly into binary:
    1. uses [rust-embed](https://github.com/pyros2097/rust-embed/) to embed static/template directories into the binary
    2. unzips static/template directories to a [temporary directory](https://github.com/Stebalien/tempfile)
    3. creates a new Rocket config (so any Rocket.toml or env vars still get parsed) which grabs templates from the temp dir

# ...why?
In the first iteration of this game I wanted to throw together some fun program in one night. Then, some people actually started playing it. The old code was super unstable and difficult to change because it was thrown together, and I wanted to learn Rust, so I decided to rewrite it.
