// SPDX-FileCopyrightText: 2024 Christina Sørensen
// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: 2023-2024 Christina Sørensen, eza contributors
// SPDX-FileCopyrightText: 2014 Benjamin Sago
// SPDX-License-Identifier: MIT
/// The version string isn’t the simplest: we want to show the version,
/// current Git hash, and compilation date when building *debug* versions, but
/// just the version for *release* versions so the builds are reproducible.
///
/// This script generates the string from the environment variables that Cargo
/// adds (<http://doc.crates.io/environment-variables.html>) and runs `git` to
/// get the SHA1 hash. It then writes the string into a file, which exa then
/// includes at build-time.
///
/// - <https://stackoverflow.com/q/43753491/3484614>
/// - <https://crates.io/crates/vergen>
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::prelude::*;

/// The build script entry point.
fn main() -> io::Result<()> {
    #![allow(clippy::write_with_newline)]

    let tagline = "wls - a patched eza for monorepo environments";
    let url = "https://github.com/eza-community/eza";

    let ver = if is_debug_build() {
        format!(
            "{}\nv{} \\1;31m(pre-release debug build!)\\0m\n\\1;4;34m{}\\0m",
            tagline,
            version_string(),
            url
        )
    } else if is_development_version() {
        format!(
            "{}\nv{} [{}] built on {} \\1;31m(pre-release!)\\0m\n\\1;4;34m{}\\0m",
            tagline,
            version_string(),
            git_hash(),
            build_date(),
            url
        )
    } else {
        format!("{}\nv{}\n\\1;4;34m{}\\0m", tagline, version_string(), url)
    };

    // We need to create these files in the Cargo output directory.
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let path = &out.join("version_string.txt");

    // Bland version text
    let mut f =
        File::create(path).unwrap_or_else(|_| panic!("{}", path.to_string_lossy().to_string()));
    writeln!(f, "{}", strip_codes(&ver))?;

    // Generate compiled LS_COLORS from dircolors source
    let ls_colors = compile_ls_colors("LS_COLORS")?;
    let ls_colors_path = &out.join("ls_colors.txt");
    let mut f = File::create(ls_colors_path)
        .unwrap_or_else(|_| panic!("{}", ls_colors_path.to_string_lossy().to_string()));
    write!(f, "{}", ls_colors)?;

    // Tell Cargo to rerun if LS_COLORS changes
    println!("cargo:rerun-if-changed=LS_COLORS");

    Ok(())
}

/// Compile a dircolors database file into LS_COLORS format.
fn compile_ls_colors(path: &str) -> io::Result<String> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Skip TERM lines
        if line.starts_with("TERM ") {
            continue;
        }

        // Parse "KEY VALUE" or "KEY VALUE # comment"
        let mut parts = line.splitn(2, char::is_whitespace);
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        let rest = match parts.next() {
            Some(r) => r.trim(),
            None => continue,
        };

        // Strip trailing comments from value
        let value = rest.split('#').next().unwrap_or("").trim();
        if value.is_empty() {
            continue;
        }

        // Convert dircolors key to LS_COLORS key
        let ls_key = match key {
            "NORMAL" | "NORM" => "no".to_string(),
            "FILE" => "fi".to_string(),
            "RESET" | "RS" => "rs".to_string(),
            "DIR" => "di".to_string(),
            "LINK" | "LNK" | "SYMLINK" => "ln".to_string(),
            "MULTIHARDLINK" => "mh".to_string(),
            "FIFO" | "PIPE" => "pi".to_string(),
            "SOCK" => "so".to_string(),
            "DOOR" => "do".to_string(),
            "BLK" | "BLOCK" => "bd".to_string(),
            "CHR" | "CHAR" => "cd".to_string(),
            "ORPHAN" => "or".to_string(),
            "MISSING" => "mi".to_string(),
            "SETUID" => "su".to_string(),
            "SETGID" => "sg".to_string(),
            "CAPABILITY" => "ca".to_string(),
            "STICKY_OTHER_WRITABLE" => "tw".to_string(),
            "OTHER_WRITABLE" => "ow".to_string(),
            "STICKY" => "st".to_string(),
            "EXEC" => "ex".to_string(),
            // Extensions: .foo -> *.foo
            k if k.starts_with('.') => format!("*{}", k),
            // Already glob patterns: *foo stays *foo
            k if k.starts_with('*') => k.to_string(),
            // Unknown keys, pass through
            k => k.to_string(),
        };

        entries.push(format!("{}={}", ls_key, value));
    }

    Ok(entries.join(":"))
}

/// Removes escape codes from a string.
fn strip_codes(input: &str) -> String {
    input
        .replace("\\0m", "")
        .replace("\\1;31m", "")
        .replace("\\1;4;34m", "")
}

/// Retrieve the project’s current Git hash, as a string.
fn git_hash() -> String {
    use std::process::Command;

    String::from_utf8_lossy(
        &Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .trim()
    .to_string()
}

/// Whether we should show pre-release info in the version string.
///
/// Both weekly releases and actual releases are --release releases,
/// but actual releases will have a proper version number.
fn is_development_version() -> bool {
    cargo_version().ends_with("-pre") || env::var("PROFILE").unwrap() == "debug"
}

/// Whether we are building in debug mode.
fn is_debug_build() -> bool {
    env::var("PROFILE").unwrap() == "debug"
}

/// Retrieves the [package] version in Cargo.toml as a string.
fn cargo_version() -> String {
    env::var("CARGO_PKG_VERSION").unwrap()
}

/// Returns the version and build parameters string.
fn version_string() -> String {
    let mut ver = cargo_version();

    let feats = nonstandard_features_string();
    if !feats.is_empty() {
        ver.push_str(&format!(" [{}]", &feats));
    }

    ver
}

/// Finds whether a feature is enabled by examining the Cargo variable.
fn feature_enabled(name: &str) -> bool {
    env::var(format!("CARGO_FEATURE_{name}"))
        .map(|e| !e.is_empty())
        .unwrap_or(false)
}

/// A comma-separated list of non-standard feature choices.
fn nonstandard_features_string() -> String {
    let mut s = Vec::new();

    if feature_enabled("GIT") {
        s.push("+git");
    } else {
        s.push("-git");
    }

    s.join(", ")
}

/// Formats the current date as an ISO 8601 string.
fn build_date() -> String {
    let now = Local::now();
    now.date_naive().format("%Y-%m-%d").to_string()
}
