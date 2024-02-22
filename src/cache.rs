use std::collections::HashMap;
use std::fs;
use std::io::Read;
use async_lock::RwLock;
use once_cell::sync::Lazy;
use std::time::SystemTime;
use crate::compress;


pub static CACHE: Lazy<RwLock<Cache>> = Lazy::new(|| {
	RwLock::new(Cache::new())
});

pub const MAX_CACHE_SIZE: usize = 1024*1024*5;

pub struct Cache {
	pub files: HashMap<String, CachedFile>,
	pub ordering: Vec<String>,
	pub size: usize,
}

impl Cache {
	pub fn new() -> Self {
		Cache { files: HashMap::new(), ordering: vec![], size: 0 }
	}
	pub async fn get(&mut self, path: &String) -> Option<Vec<u8>> {
		self.update_ordering(path.to_string());
		if self.files.contains_key(path) {
			let cf = self.files.get(path).unwrap();
			if cf.is_outdated() {
				self.remove(path);
				let cf = self.set(path).await;
				return Some(cf.content)
			} else {
				return Some(cf.content.clone())
			}
		} else {
			self.check_size();
			let cf = self.set(path).await;
			return Some(cf.content)
		}
	}
	pub async fn set(&mut self, path: &String) -> CachedFile {
		let cf = CachedFile::new(path);
		self.size += &cf.content.len();
		match self.files.insert(path.to_string(), cf) {
			Some(v) => v,
			None => self.files.get(path).unwrap().clone()
		}
	}
	pub fn remove(&mut self, path: &String) {
		let cf = self.files.get(path).unwrap();
		self.size -= cf.content.len();
		self.files.remove(path);
	}
	pub fn check_size(&mut self) {
		while self.size > MAX_CACHE_SIZE {
			let oldest = self.ordering.remove(0);
			let cf = self.files.get(&oldest).unwrap();
			self.size -= cf.content.len();
			self.files.remove(&oldest).unwrap();
		}
	}
	pub fn update_ordering(&mut self, path: String) {
		if let Some(idx) = self.ordering.iter().position(|x| x == &path) {
			self.ordering.remove(idx);
		}
		self.ordering.push(path);
	}
}

#[derive(Clone)]
pub struct CachedFile {
	path: String,
	content: Vec<u8>,
	mod_dt: SystemTime,
	hits: u64,
}

impl CachedFile {
	pub fn new(path: &String) -> Self {
		let mut f = fs::File::open(path).unwrap();
		let mut buf: Vec<u8> = Vec::new();
		f.read_to_end(&mut buf).unwrap();
		buf = compress::compress(&buf);

		let mod_dt = f.metadata().unwrap().modified().unwrap();
		CachedFile {
			path: path.to_string(),
			content: buf,
			mod_dt: mod_dt,
			hits: 1,
		}
	}
	pub fn hit(&mut self) {
		self.hits += 1;
	}
	pub fn is_outdated(&self) -> bool {
		match fs::metadata(&self.path) {
			Err(_) => return true,
			Ok(metadata) => return self.mod_dt != metadata.modified().unwrap()
		}
	}
}
