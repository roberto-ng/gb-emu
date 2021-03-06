use std::fs;
use std::io::Write;
use std::path::Path;

#[cfg(not(target_family = "wasm"))]
use directories::BaseDirs;

pub enum ConfigFile {
    LastUsedDirectory,
}

impl ConfigFile {
    pub const fn get_file_name(&self) -> &str {
        match &self {
            Self::LastUsedDirectory => "last_directory.txt",
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn save_config(config_file: ConfigFile, content: &str) -> std::io::Result<()> {
    if let Some(dirs) = BaseDirs::new() {
        let config_path = dirs.config_dir();
        let app_config_path = config_path.join("rustboy");
        let config_file_name = config_file.get_file_name();
        let config_file_path = app_config_path.join(&config_file_name);
        fs::create_dir_all(&app_config_path)?; // create dir if it not exists

        let mut file = fs::File::create(&config_file_path)?;
        write!(&mut file, "{content}")?;
    } else {
        panic!("Could not find base dirs");
    }

    Ok(())
}

#[cfg(target_family = "wasm")]
pub fn read_config(config_file: ConfigFile) -> std::io::Result<Option<String>> {
    Ok(None)
}

#[cfg(not(target_family = "wasm"))]
pub fn read_config(config_file: ConfigFile) -> std::io::Result<Option<String>> {
    if let Some(dirs) = BaseDirs::new() {
        let config_path = dirs.config_dir();
        let app_config_path = config_path.join("rustboy");
        let config_file_name = config_file.get_file_name();
        let config_file_path = app_config_path.join(&config_file_name);

        if config_file_path.exists() {
            let content = fs::read_to_string(&config_file_path)?;

            match &config_file {
                ConfigFile::LastUsedDirectory => {
                    // check if path exists
                    if Path::new(&content).exists() {
                        return Ok(Some(content));
                    }
                }
            }
        }
    }

    Ok(None)
}
