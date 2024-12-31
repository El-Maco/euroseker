use std::{fs, io::Write};

use chrono::{DateTime, Utc};

use crate::exchangerate::monitor::ExchangeRate;


pub fn read_from_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or("[]".to_string())
}

pub fn write_to_file(data: &Vec<ExchangeRate>, file_path: &str) {
    let json_data = serde_json::to_string_pretty(&data).expect("Failed to serialize data");
    let mut file = fs::File::create(file_path).expect("Failed to create file");
    file.write_all(json_data.as_bytes()).expect("Failed to write file");
}

pub struct FileStorage {
    storage_file: String,
    history: Vec<ExchangeRate>,
}

impl FileStorage {
    pub fn new(filepath: &str) -> Self {
        let mut storage = FileStorage{
            storage_file: filepath.to_owned(),
            history: vec![],
        };
        storage.load_storage();
        println!("History: {:?}", storage.history);
        storage
    }

    pub fn load_storage(&mut self) {
        let stored_data = read_from_file(&self.storage_file);

        let entries: Vec<ExchangeRate> = serde_json::from_str(&stored_data).unwrap_or_default();
        self.history = entries;
    }

    pub fn add(&mut self, exchange_rate: ExchangeRate) {
        self.history.push(exchange_rate);
        self.save();
    }

    fn save(&self) {
        write_to_file(&self.history, &self.storage_file);
    }
}
