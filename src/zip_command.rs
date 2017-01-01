// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use zip::Zip;
use errors::Result;
use errors::ResultExt;

use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::io::Read;
use std::fs;
use std::fs::DirBuilder;
use std::fs::File;
use std::process::Command;

use tempdir::TempDir;

/// Zip files using the `zip` command.
///
/// Create a temporary directory, write temp files in that directory, and then
/// calls the zip command to generate an epub file.
pub struct ZipCommand {
    command: String,
    temp_dir: TempDir,
    files: Vec<PathBuf>,
}

impl ZipCommand {
    /// Creates a new ZipCommand, using default setting to create a temporary directory.
    pub fn new() -> Result<ZipCommand> {
        let temp_dir = TempDir::new("epub")
            .chain_err(|| "could not create temporary directory")?;
        let zip = ZipCommand {
            command: String::from("zip"),
            temp_dir: temp_dir,
            files: vec!()
        };
        Ok(zip)
    }
    
    /// Creates a new ZipCommand, specifying where to create a temporary directory.
    ///
    /// # Arguments
    /// * `temp_path`: the path where a temporary directory should be created.
    pub fn new_in<P: AsRef<Path>>(temp_path: P) -> Result<ZipCommand> {
        let temp_dir = TempDir::new_in(temp_path, "epub")
            .chain_err(|| "could not create temporary directory")?;
        let zip = ZipCommand {
            command: String::from("zip"),
            temp_dir: temp_dir,
            files: vec!()
        };
        Ok(zip)
    }

    /// Set zip command to use
    pub fn command<S: Into<String>>(&mut self, command: S) -> &mut Self {
        self.command = command.into();
        self
    }
}


impl Zip for ZipCommand {
    fn write_file<P:AsRef<Path>, R: Read>(&mut self, path: P, mut content: R) -> Result<()> {
        let path = path.as_ref();
        if path.starts_with("..") || path.is_absolute() {
            bail!("file {file} refers to a path outside the temporary directory. This is verbotten!");
        }

        let dest_file = self.temp_dir.path().join(path);
        let dest_dir = dest_file.parent().unwrap();
        if !fs::metadata(dest_dir).is_ok() {
            // dir does not exist, create it
            DirBuilder::new()
                .recursive(true)
                .create(&dest_dir)
                .chain_err(|| format!("could not create temporary directory in {path}",
                                      path = dest_dir.display()))?;
        }


        let mut f = File::create(&dest_file)
            .chain_err(|| format!("could not write to temporary file {file}",
                                           file = path.display()))?;
        io::copy(&mut content, &mut f)
            .chain_err(|| format!("could not write to temporary file {file}",
                                  file = path.display()))?;
        self.files.push(path.to_path_buf());
        Ok(())
    }

    fn generate(&mut self) -> Result<Vec<u8>> {
        let mut command = Command::new(&self.command);
        command
            .current_dir(self.temp_dir.path())
            .arg("-X")
            .arg("-");
        for file in self.files.iter() {
            command.arg(format!("{}", file.display()));
        }
            
        let output = command.output()
            .chain_err(|| format!("failed to run command {name}",
                                  name = self.command))?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            bail!("command {name} didn't return succesfully: {output}",
                  name = self.command,
                  output = String::from_utf8_lossy(&output.stderr));
        }
    }
}
