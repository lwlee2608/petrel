use diameter::dictionary;
use diameter::dictionary::Dictionary;
use std::error::Error;
use std::fs;
use std::path::Path;
use url::Url;

pub fn load(filenames: Vec<String>) -> Result<Dictionary, Box<dyn Error>> {
    let mut dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);

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
    Ok(dict)
}
