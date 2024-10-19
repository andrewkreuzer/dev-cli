pub mod operations;

use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

use serde_yaml::Value;

pub async fn update(filepath: PathBuf, target: &str) -> Result<(), anyhow::Error> {
    let mut value = read_file(&filepath).await?;
    operations::walk(&mut value, target, "");
    write_file(filepath, value).await?;

    Ok(())
}

async fn read_file(filepath: &PathBuf) -> Result<Value, serde_yaml::Error> {
    let mut file = fs::File::open(filepath).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("file read error");
    let value: Value = serde_yaml::from_str(&contents)?;

    Ok(value)
}

async fn write_file(filepath: PathBuf, value: Value) -> Result<PathBuf, serde_yaml::Error> {
    let write_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(&filepath)
        .unwrap();
    serde_yaml::to_writer(write_file, &value).unwrap();

    Ok(filepath)
}
