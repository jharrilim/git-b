use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "git-b")]
#[command(author = "Joseph Harrison-Lim <josephharrisonlim@gmail.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Fuzzy git branch picker.")]
pub struct Args {
    /// Branch name to fuzzy-match and checkout directly (skips the picker).
    #[arg()]
    pub branch: Option<String>,
}
