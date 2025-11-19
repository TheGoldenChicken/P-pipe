use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

// TODO IMPORTANT: If the saved objects are to be used for python testing, perhaps change their source data location...

/// Save any serializable object to the `json_instances` folder.
/// The filename is derived from the provided `name` argument.
pub fn save_instance<T: Serialize>(obj: &T, name: &str) -> std::io::Result<()> {
    // Ensure the folder exists
    let dir = Path::new("json_instances");
    fs::create_dir_all(dir)?;

    // Build the file path
    let file_path = dir.join(format!("{}.json", name));

    // Serialize the object to JSON
    let json = serde_json::to_string_pretty(obj)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Write to file
    let mut file = File::create(file_path)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}
