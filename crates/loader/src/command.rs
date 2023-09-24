use std::{collections::HashSet, io::Result, path::Path, process::Command, str};

pub struct CargoCommand {
    pub command: Command,
}

impl CargoCommand {
    pub fn new(kind: &str) -> Self {
        let mut command = Command::new("cargo");
        command.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
        command.arg(kind);
        Self { command }
    }

    pub fn args(&mut self, args: &[&str]) -> &mut Self {
        self.command.args(args);
        self
    }

    pub fn packages(&mut self, names: &[&str]) -> &mut Self {
        self.command
            .args(names.iter().flat_map(|package_name| ["-p", package_name]));
        self
    }
}

pub fn package_dependency_count(packages: &[&str]) -> Result<usize> {
    let output = CargoCommand::new("tree")
        .args(&[
            "--edges",
            "no-dev",
            "--target",
            "wasm32-unknown-unknown",
            "--prefix",
            "none",
        ])
        .packages(packages)
        .command
        .output()?;

    // Turns out that tree might list the same dependency multiple times, so we need to deduplicate
    let mut deps = HashSet::new();

    str::from_utf8(&output.stdout)
        .expect("Failed to convert to String")
        .lines()
        // Dependencies marked with (*) have already been listed
        .filter(|line| !line.ends_with("(*)") && !line.is_empty())
        .for_each(|line| {
            deps.insert(line.trim());
        });

    Ok(deps.len())
}
