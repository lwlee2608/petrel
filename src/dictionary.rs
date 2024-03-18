use diameter::dictionary;
use std::error::Error;
use std::fs;
use std::path::Path;
use url::Url;

pub fn load(filenames: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut dict = dictionary::DEFAULT_DICT.write().unwrap();

    for filename in filenames {
        let xml = if Url::parse(&filename).is_ok() {
            log::info!("Loading dictioanry from url: {}", filename);
            reqwest::blocking::get(&filename)?.text()?
        } else if Path::new(&filename).exists() {
            log::info!("Loading dictionary from local file: {}", filename);
            fs::read_to_string(filename)?
        } else {
            return Err(format!("File not found: {}", filename).into());
        };

        dict.load_xml(&xml);
    }
    Ok(())
}
