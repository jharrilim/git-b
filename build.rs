use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use clap::CommandFactory;
use clap_mangen::Man;
use git_b_cli::Args;

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?);
    let man_dir = manifest_dir.join("man");
    let man_path = man_dir.join("git-b.1");

    fs::create_dir_all(&man_dir)?;

    let man = Man::new(Args::command());
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;
    fs::write(&man_path, buffer)?;

    println!("cargo:rerun-if-changed=cli/src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
