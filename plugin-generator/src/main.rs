#[macro_use]
extern crate log;
extern crate fern;

use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::env;
use std::collections;

const PLUGIN_CARGO_TEMPLATE: &'static str = include_str!("../templates/cargo.toml");
const TOP_CARGO_TEMPLATE: &'static str = include_str!("../templates/plugins-crate/cargo.toml");
const TOP_LIB_TEMPLATE: &'static str = include_str!("../templates/plugins-crate/lib.rs");

fn main() {
    if let Err(e) = main_possibly_errors() {
        panic!("Build error! Error: {} ({:?})", e, e);
    }
}

fn setup_logger(out_dir: &Path) -> Result<(), fern::InitError> {
    try!(fs::create_dir_all(out_dir));
    fern::init_global_logger(
        fern::OutputConfig::File(out_dir.join("output.log")),
        log::LogLevelFilter::Trace
    )
}

fn main_possibly_errors() -> io::Result<()> {
    let zaldinar_runtime_dir_str = env::var_os("CARGO_MANIFEST_DIR")
        .expect("Expected CARGO_MANIFEST_DIR, found None");

    let source_plugin_directory = Path::new(&zaldinar_runtime_dir_str).parent()
        .expect("CARGO_MANIFEST_DIR does not have a parent path.")
        .join("plugins");

    let output_dir = Path::new(&zaldinar_runtime_dir_str).parent().unwrap()
        .join("build-out");

    if let Err(e) = setup_logger(&output_dir) {
        panic!("Error setting up logging: {}", e);
    }

    let generated_plugin_directory = output_dir.join("plugin-crates");

    let zaldinar_core_path = Path::new(&zaldinar_runtime_dir_str).parent().unwrap()
        .join("zaldinar-core");

    let core_path_str = zaldinar_core_path.to_str().expect(
            "Expected valid UTF8 zaldinar-core path (for embedding in Cargo.toml), found None");

    debug!("Creating generated plugin directory: {}", generated_plugin_directory.display());
    try!(fs::create_dir_all(&generated_plugin_directory));

    let mut created_plugins = Vec::new();

    debug!("Walking {}", source_plugin_directory.display());
    for dir_entry in try!(fs::read_dir(&source_plugin_directory)) {
        let plugin_path = try!(dir_entry).path();
        if plugin_path.file_name().unwrap() == "template.rs" {
            continue; // ignore template.rs!
        }
        let created = try!(create_plugin_crate(&plugin_path, &generated_plugin_directory,
            &core_path_str));
        created_plugins.push(created);
    }

    debug!("Creating main dependency crate.");

    {

        let cumulative_crate_dir = output_dir.join("cumulative-crate");

        // Create directory
        debug!("Creating cumulative plugin crate directory: {}", cumulative_crate_dir.display());
        try!(fs::create_dir_all(&cumulative_crate_dir));

        // Set of names which have already been used for importing
        // Because we take the last string in a plugin name split by `-`, this is needed
        // to prevent names like `thing1-cmds` and `thing2-cmds` to conflict, just as an example.
        let mut taken_import_names = collections::HashSet::new();
        // List of dependency lines to insert into Cargo.toml
        let mut dependency_lines = Vec::new();
        // List of `extern crate ...` lines
        let mut extern_crate_lines = Vec::new();
        // List of `xxx::register(register);` lines
        let mut register_lines = Vec::new();

        // We use the zaldinar name for zaldinar-core
        taken_import_names.insert("zaldinar".to_string());

        for (crate_name, path) in created_plugins {
            dependency_lines.push(format!("[dependencies.{}]", crate_name));
            dependency_lines.push(format!("path = \"{}\"", path.display()));

            let imported_name = { // This calculation could probably be done more efficiently.
                // any string.split() will at least have one element
                let name_without_additions = crate_name.split("-").next().unwrap();
                let mut temp_name = name_without_additions.to_string();
                // TODO: this could possibly overflow, some check for that?
                let mut addition = 0u32;
                while !taken_import_names.insert(temp_name.clone()) {
                    // Continue incrementing an integer added to the end of the name until the
                    // name is no longer taken.
                    temp_name = format!("{}{}", name_without_additions, addition);
                    addition = addition + 1;
                }
                temp_name
            };

            extern_crate_lines.push(format!("extern crate \"{}\" as {};",
                                            crate_name, imported_name));
            register_lines.push(format!("{}::register(register);", imported_name));
        }

        {
            let cargo_toml_contents = TOP_CARGO_TEMPLATE
                    .replace("{{dependency_lines}}", &dependency_lines.connect("\n"))
                    .replace("{{zaldinar_core_path}}", &core_path_str);

            let cargo_toml_path = cumulative_crate_dir.join("Cargo.toml");
            try!(write_file(&cargo_toml_path, &cargo_toml_contents));
        }

        {
            let lib_rs_contents = TOP_LIB_TEMPLATE
                    .replace("{{extern_crate_lines}}", &extern_crate_lines.connect("\n"))
                    .replace("{{register_lines}}", &register_lines.connect("\n"));

            let src_path = cumulative_crate_dir.join("src");
            debug!("Creating source directory {}", src_path.display());
            try!(fs::create_dir_all(&src_path));

            try!(write_file(&src_path.join("lib.rs"), &lib_rs_contents));
        }
    }

    return Ok(());
}

fn create_plugin_crate(path: &Path, output_dir: &Path, core_path_str: &str)
        -> io::Result<(String, PathBuf)> {
    debug!("Reading plugin source file: {}", path.display());

    let contents = {
        let mut stream = try!(fs::File::open(path));
        let mut buf = String::new();
        try!(stream.read_to_string(&mut buf));
        buf
    };

    let mut dependency_lines = Vec::new();

    for line in contents.lines_any() {
        let split_iter = line.split("//! depends: ");
        // If the line has `//! depends: `, .skip(1) will remove all content before the
        // `//! depends: `. If it doesn't, then .skip(1) will leave an empty iterator, and this if
        // let statement won't run due to .next() returning None.
        if let Some(dependency_line) = split_iter.skip(1).next() {
            dependency_lines.push(dependency_line);
        }
    }

    let name = {
        let name_os = path.file_stem().unwrap();
        let temp_name = name_os.to_str().expect("Expected UTF8 file name for plugin, found None");
        format!("zaldinar-plugin-{}", temp_name)
    };
    let output_directory = output_dir.join(&name);

    debug!("Creating plugin crate from {{source: {:?}, name: {:?}, dependencies: {:?}}} at {:?}",
        path.display(), name, dependency_lines, output_directory.display());


    // Create directory
    debug!("Creating crate directory: {}", output_directory.display());
    try!(fs::create_dir_all(&output_directory));

    { // Scope for writing Cargo.toml

        // TODO: This could probably be done more efficiently
        let dependencies_contents = dependency_lines.connect("\n");
        let cargo_contents = PLUGIN_CARGO_TEMPLATE.replace("{{name}}", &name)
                                .replace("{{dependencies}}", &dependencies_contents)
                                .replace("{{zaldinar_core_path}}", core_path_str);

        // Write to the file
        try!(write_file(&output_directory.join("Cargo.toml"), &cargo_contents));
    }

    { // Scope for writing lib.rs
        let src_path = output_directory.join("src");

        debug!("Creating source directory {}", src_path.display());
        try!(fs::create_dir_all(&src_path));

        // This is just writing the contents which were originally read from the plugin.rs file.
        try!(write_file(&src_path.join("lib.rs"), &contents));
    }

    return Ok((name, output_directory));
}

fn write_file(path: &Path, contents: &str) -> io::Result<()> {
    if try!(is_same(path, contents)) {
        debug!("Skipping writing to {}, as file contents are same", path.display());
        return Ok(());
    }

    // create a parent directory, if it has a parent
    if let Some(parent) = path.parent() {
        debug!("Creating directory {}", parent.display());
        try!(fs::create_dir_all(parent));
    }

    debug!("Writing to {}", path.display());
    let mut stream = try!(fs::File::create(path));
    try!(stream.write(contents.as_bytes()));

    return Ok(());
}

fn is_same(path: &Path, expected_contents: &str) -> io::Result<bool> {
    debug!("Checking differences for {}", path.display());
    let metadata = match fs::metadata(path) {
        Ok(v) => v,
        Err(e) => if e.kind() == io::ErrorKind::NotFound {
            return Ok(false); // File does not exist
        } else {
            return Err(e);
        }
    };
    if metadata.len() != expected_contents.len() as u64 {
        return Ok(false);
    }
    let file_stream = try!(fs::File::open(path));
    // this gets if any bytes in the file are different from bytes in the expected contents,
    // without holding the file in memory.
    for (read_byte, expected_byte) in file_stream.bytes().zip(expected_contents.bytes()) {
        if try!(read_byte) != expected_byte {
            return Ok(false);
        }
    }
    return Ok(true);
}
