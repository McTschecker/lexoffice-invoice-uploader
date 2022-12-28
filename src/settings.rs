use log::{info};
use serde::{Deserialize, Serialize};
use text_io::read;
use confy;
use confy::ConfyError;

#[derive(Serialize, Deserialize, Debug)]
pub struct PrefixConfig {
    pub prefix: String,
    pub path: String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api_key: String,
    pub prefixes: Option<Vec<PrefixConfig>>
}

impl Config {
    pub fn validate(&self) -> bool {
        return self.api_key.len() > 15;
    }
}

impl Default for Config {
    fn default() -> Self { Self { api_key: update_api_key(), prefixes: Some(vec![]) } }
}

pub fn load_settings() -> Result<Config, confy::ConfyError> {
    let mut cfg: Config = confy::load("lexUploadConfig", None)?;

    // initializes the prefixes if they are empty
    init_prefixes(&mut cfg).expect("Failed to initialize prefixes");

    Ok(cfg)
}

fn init_prefixes(cfg: &mut Config) -> Result<(), confy::ConfyError> {
    if cfg.prefixes.is_none() {
        cfg.prefixes = Some(vec![]);
    }
    confy::store("lexUploadConfig", None, &cfg)
}

pub fn update_settings(){
    let new_config = Config {
        api_key: update_api_key(),
        prefixes: Some(vec![])
    };

    confy::store("lexUploadConfig", None, new_config).unwrap();
}

fn update_api_key() -> String {
    info!("Getting api key from user!");
    println!("Please enter your API KEY and confirm with enter: ");
    let api_key: String = read!("{}\n");
    println!("You entered: {}", api_key);
    api_key
}

#[allow(dead_code)]
pub fn get_prefix_path(prefix: String) -> Result<(), ConfyError> {
    let mut config = load_settings()?;
    println!("Got a new Path: {}. \n Please enter the corresponding Folder (e.g. alias) for \
        the alias folder", prefix);

    let user_input = read!("{}\n");

    // add the user input to the path lists
    let new_prefix = PrefixConfig {
        prefix: prefix,
        path: user_input,
    };
    let mut prefixes = match config.prefixes {
        Some(prefixes) => prefixes,
        None => vec![],
    };
    prefixes.push(new_prefix);
    config.prefixes = Some(prefixes);
    confy::store("lexUploadConfig", None, config)
}