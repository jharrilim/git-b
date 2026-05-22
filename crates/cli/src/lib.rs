pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "git-b")]
#[command(author = "Joseph Harrison-Lim <josephharrisonlim@gmail.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "Fuzzy git branch picker.",
    long_about = "Interactive fuzzy finder for git branches, sorted by recent commit date.\n\n\
                  Without arguments, opens a picker to choose a branch. With a branch name, \
                  checks out the first fuzzy match. Use -b to create and checkout a new branch."
)]
pub struct Args {
    /// Create and checkout a new branch (`git checkout -b`).
    #[arg(short = 'b')]
    pub new_branch: Option<String>,

    /// Branch name to fuzzy-match and checkout directly (skips the picker).
    #[arg()]
    pub branch: Option<String>,

    /// Disable colored branch listing in the picker.
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Set field colors as NAME:HASH:SUBJECT (e.g. cyan:yellow:white).
    #[arg(long = "color", value_name = "NAME:HASH:SUBJECT")]
    pub color: Option<String>,
}
