zaldinar
========

A concurrently-run plugin-enabled IRC bot written in Rust.

Rust is a language designed to run at C-level speeds, while maintaining type and memory safety at standards above even what higher level languages have to offer.

Zaldinar is an IRC bot written in rust, designed to showcase some rust's features and speed, and also just as a general experiment.

Zaldinar may only be used while complying to the license terms in the `LICENSE` file.

Repository layout
===

Zaldinar is split into 4 parts:

- `zaldinar-irclib` is the basic irc parsing library, which is used to communicate with the irc server
- `zaldinar-core` is the majority of zaldinar's inner workings. It contains all of the data types and events used by plugins and zaldinar itself.
- `plugins` is a folder which contains a set of separate "plugin" rust files. Each plugin is generated into a full cargo crate by the `plugin-generator` application.
- `zaldinar-runtime` depends on and builds everything together. It contains the dispatch code to run plugins in separate threads, the startup code to initialize everything, and compiles to the main `zaldinar` binary.

Building zaldinar
===

In order to build zaldinar, first you need to build and run the `plugin-generator` crate. A simple `cargo run --release` in the `plugin-generator` directory will generate all the plugin crates you need into the `build-out` folder.

Then, after the plugin generator has run, you can just use `cargo build` in the `zaldinar-runtime` directory to build zaldinar! The plugin generator will need to be re-run every time you change plugins any of the plugin files, but it is very fast after compiling for the first time.

I recommend creating a shell script for building zaldinar similar to the following in order to automatically run the plugin generator and build zaldinar:

```bash
#!/bin/bash
ZALDINAR_PATH="/home/daboross/Projects/Rust/zaldinar/"

cd "${ZALDINAR_PATH}/plugin-generator" &&
cargo run --release &&
cd "${ZALDINAR_PATH}/zaldinar-runtime" &&
if [[ ! -z "$@" ]]; then
    cargo build "$@"
else
    cargo build
fi
```

Zaldinar feature highlights
===

TODO: Make these!
