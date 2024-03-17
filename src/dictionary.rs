use diameter::dictionary;
use std::error::Error;
use std::fs;

pub fn load(filenames: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut dict = dictionary::DEFAULT_DICT.write().unwrap();
    for filename in filenames {
        let xml = fs::read_to_string(filename)?;
        dict.load_xml(&xml);
    }
    Ok(())
}
