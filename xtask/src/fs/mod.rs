// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use color_eyre::eyre::anyhow;
use color_eyre::Result;
use fatfs::{FormatVolumeOptions, FsOptions};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub enum FileSystemEntry {
    Directory,
    File(PathBuf),
    Buffer(Vec<u8>),
}

pub struct FileSystem(BTreeMap<String, FileSystemEntry>);

impl FileSystem {
    pub fn new() -> Self {
        FileSystem(BTreeMap::new())
    }

    pub fn create_directory(&mut self, path: &str) -> Result<()> {
        for segment in PathIter::new(path) {
            if segment.len() == path.len() {
                self.0
                    .insert(segment.to_string(), FileSystemEntry::Directory);
                break;
            } else {
                let entry = self
                    .0
                    .entry(segment.to_string())
                    .or_insert(FileSystemEntry::Directory);

                if !matches!(entry, FileSystemEntry::Directory) {
                    return Err(anyhow!("Path segment {} is not a directory", segment));
                }
            }
        }

        Ok(())
    }

    pub fn create_file(&mut self, path: &str, file: PathBuf) -> Result<()> {
        for segment in PathIter::new(path) {
            if segment.len() == path.len() {
                let old = self
                    .0
                    .insert(segment.to_string(), FileSystemEntry::File(file));

                if matches!(old, Some(FileSystemEntry::Directory)) {
                    self.purge_children(segment);
                }

                break;
            } else {
                let entry = self
                    .0
                    .entry(segment.to_string())
                    .or_insert(FileSystemEntry::Directory);

                if !matches!(entry, FileSystemEntry::Directory) {
                    return Err(anyhow!("Path segment {} is not a directory", segment));
                }
            }
        }

        Ok(())
    }

    pub fn create_buffer(&mut self, path: &str, buffer: Vec<u8>) -> Result<()> {
        for segment in PathIter::new(path) {
            if segment.len() == path.len() {
                let old = self
                    .0
                    .insert(segment.to_string(), FileSystemEntry::Buffer(buffer));

                if matches!(old, Some(FileSystemEntry::Directory)) {
                    self.purge_children(segment);
                }

                break;
            } else {
                let entry = self
                    .0
                    .entry(segment.to_string())
                    .or_insert(FileSystemEntry::Directory);

                if !matches!(entry, FileSystemEntry::Directory) {
                    return Err(anyhow!("Path segment {} is not a directory", segment));
                }
            }
        }

        Ok(())
    }

    pub fn entry(&self, mut path: &str) -> Option<&FileSystemEntry> {
        if !path.is_empty() && path.as_bytes()[0] == b'/' {
            path = &path[1..];
        }

        self.0.get(path)
    }

    pub fn entry_mut(&mut self, mut path: &str) -> Option<&mut FileSystemEntry> {
        if !path.is_empty() && path.as_bytes()[0] == b'/' {
            path = &path[1..];
        }

        self.0.get_mut(path)
    }

    pub fn is_directory(&self, path: &str) -> bool {
        matches!(self.entry(path), Some(FileSystemEntry::Directory))
    }

    pub fn is_file(&self, path: &str) -> bool {
        matches!(self.entry(path), Some(FileSystemEntry::File(_)))
    }

    pub fn is_buffer(&self, path: &str) -> bool {
        matches!(self.entry(path), Some(FileSystemEntry::Buffer(_)))
    }

    pub fn exists(&self, path: &str) -> bool {
        self.entry(path).is_some()
    }

    pub fn delete(&mut self, mut path: &str) -> Option<FileSystemEntry> {
        if !path.is_empty() && path.as_bytes()[0] == b'/' {
            path = &path[1..];
        }

        let result = self.0.remove(path);

        if matches!(result, Some(FileSystemEntry::Directory)) {
            self.purge_children(path);
        }

        result
    }

    pub fn write_fat_image(&self, path: &Path, label: [u8; 11]) -> Result<()> {
        const MB: u64 = 1024 * 1024;

        let mut size = 0;
        for entry in self.0.values() {
            match entry {
                FileSystemEntry::Directory => {}
                FileSystemEntry::File(file) => size += fs::metadata(file)?.len(),
                FileSystemEntry::Buffer(buffer) => size += buffer.len() as u64,
            }
        }

        let size = ((size + 1024 * 64 - 1) / MB + 1) * MB + MB;

        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.set_len(size)?;

        let options = FormatVolumeOptions::new().volume_label(label);
        fatfs::format_volume(&mut file, options)?;

        let fat = fatfs::FileSystem::new(file, FsOptions::new())?;
        let root = fat.root_dir();

        for dir in self.0.iter().filter_map(|(k, v)| {
            if matches!(v, FileSystemEntry::Directory) {
                Some(k)
            } else {
                None
            }
        }) {
            root.create_dir(dir)?;
        }

        for (path, entry) in self
            .0
            .iter()
            .filter(|(_, v)| matches!(v, FileSystemEntry::File(_) | FileSystemEntry::Buffer(_)))
        {
            let mut file = root.create_file(path)?;
            file.truncate()?;

            match entry {
                FileSystemEntry::File(path) => {
                    io::copy(&mut File::open(path)?, &mut file)?;
                }
                FileSystemEntry::Buffer(buffer) => {
                    file.write_all(buffer)?;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub fn write_to_path(&self, path: &Path) -> Result<()> {
        for (sub, entry) in &self.0 {
            let target = path.join(sub);
            match entry {
                FileSystemEntry::Directory => fs::create_dir(target)?,
                FileSystemEntry::File(source) => {
                    fs::copy(source, target)?;
                }
                FileSystemEntry::Buffer(buffer) => fs::write(target, buffer)?,
            }
        }

        Ok(())
    }

    fn purge_children(&mut self, path: &str) {
        self.0.retain(|k, _| !k.starts_with(path))
    }
}

struct PathIter<'a> {
    path: &'a str,
    index: usize,
}

impl<'a> PathIter<'a> {
    pub fn new(path: &'a str) -> Self {
        if !path.is_empty() && path.as_bytes()[0] == b'/' {
            PathIter {
                path: &path[1..],
                index: 0,
            }
        } else {
            PathIter { path, index: 0 }
        }
    }
}

impl<'a> Iterator for PathIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.path.len() {
            return None;
        }

        match self.path[self.index..].find('/') {
            Some(idx) => {
                self.index += idx + 1;
                Some(&self.path[..self.index - 1])
            }
            None => {
                self.index = self.path.len();
                Some(self.path)
            }
        }
    }
}
