use crate::constants::DATA_FILENAME;
use crate::models::AppData;
use std::fs;
use std::path::Path;

pub fn load_data() -> Option<AppData> {
    if Path::new(DATA_FILENAME).exists() {
        let file = fs::File::open(DATA_FILENAME).ok()?;
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).ok()
    } else {
        None
    }
}

pub fn save_data(data: &AppData) -> std::io::Result<()> {
    let file = fs::File::create(DATA_FILENAME)?;
    let writer = std::io::BufWriter::new(file);

    serde_json::to_writer_pretty(writer, data)?; // エラーを出す可能性
    Ok(())
}
