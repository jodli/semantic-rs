use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;

use cargo_toml::Manifest;
use regex::Regex;

#[derive(Debug)]
pub enum TomlError {
    Parse(&'static str),
    Io(Error),
}

fn read_manifest(file: &str) -> Option<Manifest> {
    match toml::from_str(&file) {
        Ok(manifest) => Some(manifest),
        Err(_) => None,
    }
}

fn read_workspace(file: &str) -> Option<Vec<String>> {
    match read_manifest(file) {
        Some(manifest) => match manifest.workspace {
            Some(workspace) => Some(workspace.members),
            None => None,
        },
        None => None,
    }
}

pub fn read_version(file: &str) -> Option<String> {
    match read_manifest(file) {
        Some(manifest) => match manifest.package {
            Some(package) => Some(package.version).filter(|v| !v.is_empty()),
            None => None,
        },
        None => None,
    }
}

pub fn file_with_new_version(file: String, new_version: &str) -> String {
    let re = Regex::new(r#"version\s=\s"\d+\.\d+\.\d+""#).unwrap();
    let new_version = format!("version = \"{}\"", new_version);
    re.replace(&file, &new_version[..]).to_string()
}

pub fn read_from_file(crate_dir: &str, package: &str) -> Result<Vec<String>, TomlError> {
    let file_path = Path::new(&crate_dir).join("Cargo.toml");
    let cargo_file = match read_cargo_toml(&file_path) {
        Ok(buffer) => buffer,
        Err(err) => return Err(TomlError::Io(err)),
    };

    let mut versions = vec![];

    if let Some(workspaces) = read_workspace(&cargo_file) {
        let workspaces = workspaces
            .into_iter()
            .filter(|workspace| {
                if package == "all" {
                    true
                } else {
                    workspace == package
                }
            })
            .collect::<Vec<String>>();
        for workspace in workspaces {
            versions.append(&mut read_from_file(
                Path::new(&crate_dir)
                    .join(workspace)
                    .to_str()
                    .expect("could not build path to workspace"),
                package,
            )?);
        }
    }
    if let Some(version) = read_version(&cargo_file) {
        versions.push(version);
    }
    Ok(versions)
}

pub fn write_new_version(crate_dir: &str, package: &str, new_version: &str) -> Result<(), Error> {
    let file_path = Path::new(&crate_dir).join("Cargo.toml");
    let cargo_file = read_cargo_toml(&file_path)?;

    if let Some(workspaces) = read_workspace(&cargo_file) {
        let workspaces = workspaces
            .into_iter()
            .filter(|workspace| {
                if package == "all" {
                    true
                } else {
                    workspace == package
                }
            })
            .collect::<Vec<String>>();
        for workspace in workspaces {
            write_new_version(
                Path::new(&crate_dir)
                    .join(workspace)
                    .to_str()
                    .expect("could not build path to workspace"),
                package,
                new_version,
            )?;
        }
    }
    let new_cargo_file = file_with_new_version(cargo_file, new_version);
    let mut handle = OpenOptions::new().read(true).write(true).open(file_path)?;
    handle.write_all(new_cargo_file.as_bytes())
}

fn read_cargo_toml(file_path: &Path) -> Result<String, Error> {
    let mut handle = match File::open(file_path) {
        Ok(handle) => handle,
        Err(err) => return Err(err),
    };

    let mut buffer = String::new();
    match handle.read_to_string(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    extern crate regex;
    extern crate toml;

    use super::*;

    fn example_file() -> String {
        "[package]
    name = \"semantic-rs\"
    version = \"0.1.0\"
    authors = [\"Jan Schulte <hello@unexpected-co.de>\"]
    [dependencies]
    term = \"0.2\"
    toml = \"0.1\""
            .to_string()
    }

    fn example_file_without_version() -> String {
        "[package]
    name = \"semantic-rs\"
    authors = [\"Jan Schulte <hello@unexpected-co.de>\"]
    [dependencies]
    term = \"0.2\"
    toml = \"0.1\""
            .to_string()
    }

    #[test]
    fn read_version_number() {
        let version_str = read_version(&example_file());
        assert_eq!(version_str, Some("0.1.0".into()));
    }

    #[test]
    fn read_file_without_version_number() {
        let version_str = read_version(&example_file_without_version());
        assert_eq!(version_str, None);
    }

    #[test]
    fn write_new_version_number() {
        let new_toml_file = file_with_new_version(example_file(), "0.2.0");
        let expected_file = "[package]
    name = \"semantic-rs\"
    version = \"0.2.0\"
    authors = [\"Jan Schulte <hello@unexpected-co.de>\"]
    [dependencies]
    term = \"0.2\"
    toml = \"0.1\""
            .to_string();
        assert_eq!(new_toml_file, expected_file);
    }
}
