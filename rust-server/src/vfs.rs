use std::collections::HashMap;

use crate::buffer::Buffer;

pub struct File {
    name: String,
    r#tyte: String,
    version: i32,
    content: Buffer,
}

impl File {
    pub fn new(name: String, r#tyte: String, version: i32, content: Buffer) -> Self {
        Self {
            name,
            r#tyte,
            version,
            content,
        }
    }

	pub fn version(&self) -> i32 {
		self.version
	}

    pub fn content_ref(&self) -> &Buffer {
        &self.content
    }
}

pub struct Vfs {
    files: HashMap<String, File>,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            files: Default::default(),
        }
    }

    pub fn get(&self, path: &str) -> &File {
        &self.files[path]
    }

    pub fn add(&mut self, file: File) {
        self.files.insert(file.name.clone(), file);
    }

    pub fn update(&mut self, name: &str, version: i32, updater: impl Fn(&mut Buffer)) {
        let mut file = self.files.get_mut(name).unwrap();
        file.version = version;
        updater(&mut file.content)
    }

    pub fn all_files(&self) -> std::collections::hash_map::Values<String, File> {
        self.files.values()
    }

    pub fn all_files_mut(&mut self) -> std::collections::hash_map::ValuesMut<String, File> {
        self.files.values_mut()
    }
}
