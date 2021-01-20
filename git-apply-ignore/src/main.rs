extern crate ignore;

use std::error::Error;
use std::fs;
use std::process::Command;

use ignore::gitignore::GitignoreBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let homeholder = String::from_utf8(Command::new("git").arg("top-name").output()?.stdout)?;
    let home = homeholder.trim();
    let mut gitignore = GitignoreBuilder::new(home);
    gitignore.add(".gitignore");
    let gitignore = gitignore.build()?;
    let output = Command::new("git").arg("ls-files").output()?;
    output
        .stdout
        .split(|v| v == &b'\n')
        .filter(|v| !v.is_empty())
        .map(|x| String::from_utf8(x.to_vec()).unwrap())
        .filter(|p| {
            gitignore
                .matched_path_or_any_parents(
                    p,
                    fs::metadata(&p).map(|v| v.is_dir()).unwrap_or(false),
                )
                .is_ignore()
        })
        .for_each(|p| {
            Command::new("git")
                .arg("rm")
                .arg("--cached")
                .arg(&p)
                .output()
                .map_or_else(
                    |_| eprintln!("Failed to remove `{}` from index.", &p),
                    |_| eprintln!("Removing `{}` from index.", &p),
                )
        });
    Ok(())
}
