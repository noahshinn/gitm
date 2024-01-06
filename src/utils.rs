use std::path::Path;
use std::process::Command;

pub fn does_valid_git_dir_exist() -> Result<bool, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let path = Path::new(cwd.as_path());
    if !path.exists() {
        return Ok(false);
    } else if !path.is_dir() {
        return Ok(false);
    }
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(cwd)
        .output();
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Ok(false);
            }
            let output = String::from_utf8(output.stdout)?;
            return Ok(output.trim() == "true");
        }
        Err(_) => return Ok(false),
    }
}

pub fn does_command_exist(command: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match Command::new("which").arg(command).output() {
        Ok(output) => Ok(output.status.success() && !String::from_utf8(output.stdout)?.is_empty()),
        Err(err) => Err(Box::new(err)),
    }
}
