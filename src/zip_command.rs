// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::zip::Zip;

use std::fs;
use std::fs::DirBuilder;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use eyre::bail;
use eyre::Context;
use eyre::Result;

/// Zip files using the system `zip` command.
///
/// Create a temporary directory, write temp files in that directory, and then
/// calls the zip command to generate an epub file.
///
/// This method will fail if `zip` (or the alternate specified command) is not installed
/// on the user system.
///
/// Note that these takes care of adding the mimetype (since it must not be deflated), it
/// should not be added manually.
pub struct ZipCommand {
    command: String,
    temp_dir: tempfile::TempDir,
    files: Vec<PathBuf>,
}

impl ZipCommand {
    /// Creates a new ZipCommand, using default setting to create a temporary directory.
    pub fn new() -> Result<ZipCommand> {
        let temp_dir = tempfile::TempDir::new().wrap_err("could not create temporary directory")?;
        let zip = ZipCommand {
            command: String::from("zip"),
            temp_dir,
            files: vec![],
        };
        Ok(zip)
    }

    /// Creates a new ZipCommand, specifying where to create a temporary directory.
    ///
    /// # Arguments
    /// * `temp_path`: the path where a temporary directory should be created.
    pub fn new_in<P: AsRef<Path>>(temp_path: P) -> Result<ZipCommand> {
        let temp_dir =
            tempfile::TempDir::new_in(temp_path).wrap_err("could not create temporary directory")?;
        let zip = ZipCommand {
            command: String::from("zip"),
            temp_dir,
            files: vec![],
        };
        Ok(zip)
    }

    /// Set zip command to use (default: "zip")
    pub fn command<S: Into<String>>(&mut self, command: S) -> &mut Self {
        self.command = command.into();
        self
    }

    /// Test that zip command works correctly (i.e program is installed)
    pub fn test(&self) -> Result<()> {
        let output = Command::new(&self.command)
            .current_dir(self.temp_dir.path())
            .arg("-v")
            .output()
            .wrap_err_with(|| format!("failed to run command {name}", name = self.command))?;
        if !output.status.success() {
            bail!(
                "command {name} didn't return successfully: {output}",
                name = self.command,
                output = String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    /// Adds a file to the temporary directory
    fn add_to_tmp_dir<P: AsRef<Path>, R: Read>(&mut self, path: P, mut content: R) -> Result<()> {
        let dest_file = self.temp_dir.path().join(path.as_ref());
        let dest_dir = dest_file.parent().unwrap();
        if fs::metadata(dest_dir).is_err() {
            // dir does not exist, create it
            DirBuilder::new()
                .recursive(true)
                .create(dest_dir)
                .wrap_err_with(|| {
                    format!(
                        "could not create temporary directory in {path}",
                        path = dest_dir.display()
                    )
                })?;
        }

        let mut f = File::create(&dest_file).wrap_err_with(|| {
            format!(
                "could not write to temporary file {file}",
                file = path.as_ref().display()
            )
        })?;
        io::copy(&mut content, &mut f).wrap_err_with(|| {
            format!(
                "could not write to temporary file {file}",
                file = path.as_ref().display()
            )
        })?;
        Ok(())
    }
}

impl Zip for ZipCommand {
    fn write_file<P: AsRef<Path>, R: Read>(&mut self, path: P, content: R) -> Result<()> {
        let path = path.as_ref();
        if path.starts_with("..") || path.is_absolute() {
            bail!(
                "file {} refers to a path outside the temporary directory. This is \
                   verbotten!",
                path.display()
            );
        }

        self.add_to_tmp_dir(path, content)?;
        self.files.push(path.to_path_buf());
        Ok(())
    }

    fn generate<W: Write>(&mut self, mut to: W) -> Result<()> {
        // First, add mimetype and don't compress it
        self.add_to_tmp_dir("mimetype", b"application/epub+zip".as_ref())?;
        let output = Command::new(&self.command)
            .current_dir(self.temp_dir.path())
            .arg("-X0")
            .arg("output.epub")
            .arg("mimetype")
            .output()
            .wrap_err_with(|| format!("failed to run command {name}", name = self.command))?;
        if !output.status.success() {
            bail!(
                "command {name} didn't return successfully: {output}",
                name = self.command,
                output = String::from_utf8_lossy(&output.stderr)
            );
        }

        let mut command = Command::new(&self.command);
        command
            .current_dir(self.temp_dir.path())
            .arg("-9")
            .arg("output.epub");
        for file in &self.files {
            command.arg(format!("{}", file.display()));
        }

        let output = command
            .output()
            .wrap_err_with(|| format!("failed to run command {name}", name = self.command))?;
        if output.status.success() {
            let mut f = File::open(self.temp_dir.path().join("output.epub"))
                .wrap_err("error reading temporary epub file")?;
            io::copy(&mut f, &mut to).wrap_err("error writing result of the zip command")?;
            Ok(())
        } else {
            bail!(
                "command {name} didn't return successfully: {output}",
                name = self.command,
                output = String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
fn zip_creation() {
    ZipCommand::new().unwrap();
}

#[test]
fn zip_ok() {
    let command = ZipCommand::new().unwrap();
    let res = command.test();
    assert!(res.is_ok());
}

#[test]
fn zip_not_ok() {
    let mut command = ZipCommand::new().unwrap();
    command.command("xkcodpd");
    let res = command.test();
    assert!(res.is_err());
}
