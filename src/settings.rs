use log::{info};
use serde::{Deserialize, Serialize};
use text_io::read;
use confy;
use confy::ConfyError;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrefixConfig {
    pub prefix: String,
    pub path: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Customer{
    pub customer_id: String,
    pub customer_adress: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api_key: String,
    pub prefixes: Option<Vec<PrefixConfig>>,
    pub customers: Option<Vec<Customer>>,
}

impl Config {
    pub fn validate(&self) -> bool {
        return self.api_key.len() > 15;
    }
}

impl Default for Config {
    fn default() -> Self { Self { api_key: update_api_key(), prefixes: Some(vec![]), customers: Some(vec![]) } }
}

pub fn load_settings() -> Result<Config, confy::ConfyError> {
    let mut cfg: Config = confy::load_path("lexUploadConfig.yaml")?;

    // initializes the prefixes if they are empty
    init_prefixes(&mut cfg).expect("Failed to initialize prefixes");
    init_customers(&mut cfg).expect("Failed to initialize customers");
    Ok(cfg)
}

fn init_prefixes(cfg: &mut Config) -> Result<(), confy::ConfyError> {
    if cfg.prefixes.is_none() {
        cfg.prefixes = Some(vec![]);
    }
    confy::store_path("lexUploadConfig.yaml", &cfg)
}

fn init_customers(cfg: &mut Config) -> Result<(), confy::ConfyError> {
    if cfg.customers.is_none() {
        cfg.customers = Some(vec![]);
    }
    confy::store_path("lexUploadConfig.yaml", &cfg)
}

pub fn update_settings(){
    let new_config = Config {
        api_key: update_api_key(),
        prefixes: Some(vec![]),
        customers: Some(vec![]),

    };

    confy::store_path("lexUploadConfig.yaml", new_config).unwrap();
}

fn update_api_key() -> String {
    info!("Getting api key from user!");
    println!("Please enter your API KEY and confirm with enter: ");
    let api_key: String = read!("{}\n");
    println!("You entered: {}", api_key);
    api_key
}

pub fn get_prefix_path(prefix: String) -> Result<String, ConfyError> {
    let mut config = load_settings()?;
    println!("Got a new Path: {}. \n Please enter the corresponding Folder (e.g. alias) for \
        the alias folder", prefix);

    let user_input: String = read!("{}\n");
    let result: Result<String, ConfyError> = Ok(user_input.clone());
    // add the user input to the path lists
    let new_prefix = PrefixConfig {
        prefix,
        path: user_input,
    };
    let mut prefixes = match config.prefixes {
        Some(prefixes) => prefixes,
        None => vec![],
    };
    prefixes.push(new_prefix);
    config.prefixes = Some(prefixes);
    confy::store_path("lexUploadConfig.yaml", config)?;
    result
}

fn get_customer_id(adress: &String) -> Result<String, ConfyError> {
    let mut config = load_settings()?;
    println!("Got a new Adress: {}. \n Please enter the corresponding Customer id from lexoffice", adress);

    let user_input: String = read!("{}\n");
    let result: Result<String, ConfyError> = Ok(user_input.clone());

    // validate the input is a uuid v4
    let uuid = Uuid::parse_str(&user_input);
    if uuid.is_err() {
        println!("The input is not a valid uuid v4. Please try again!");
        return get_customer_id(adress);
    }

    // add the user input to the path lists
    let new_customer = Customer {
        customer_id: user_input,
        customer_adress: adress.clone(),
    };
    let mut customers = match config.customers {
        Some(customers) => customers,
        None => vec![],
    };
    customers.push(new_customer);
    config.customers = Some(customers);
    confy::store_path("lexUploadConfig.yaml", config)?;
    result
}

impl Config {
    pub fn get_path(&mut self, prefix: &str) -> Result<String, ConfyError> {
        let prefixes = match &self.prefixes {
            Some(prefixes) => prefixes,
            None => {
                init_prefixes(self).expect("Failed to initialize prefixes");
                get_prefix_path(prefix.to_string()).expect("Failed to get path");
                // update self with new prefixes
                let new_config = load_settings().expect("Failed to load new config");
                self.prefixes = new_config.prefixes.clone();
                self.prefixes.as_ref().unwrap()
            },
        };

        for prefix_config in prefixes {
            if prefix_config.prefix == prefix {
                return Ok(prefix_config.path.clone());
            }
        }

        get_prefix_path(prefix.to_string()).expect("Failed to get path");
        // update self with new prefixes
        let new_config = load_settings().expect("Failed to load new config");
        self.prefixes = new_config.prefixes;
        // recursive call to get the path to simplify the code - should not be infinite
        self.get_path(prefix)

    }

    pub fn get_customer_id(&mut self, address: &String) -> Result<String, ConfyError> {
        let ids = match &self.customers {
            Some(customers) => customers,
            None => {
                init_customers(self).expect("Failed to initialize customers");
                get_customer_id(address).expect("Failed to get customer id");
                // update self with new prefixes
                let new_config = load_settings().expect("Failed to load new config");
                self.customers = new_config.customers.clone();
                self.customers.as_ref().unwrap()
            },
        };

        for customer in ids {
            if &customer.customer_adress == address {
                return Ok(customer.customer_id.clone());
            }
        }

        // customer id is not in the list
        get_customer_id(address).expect("Failed to get path");
        // update self with new prefixes
        let new_config = load_settings().expect("Failed to load new config");
        self.customers = new_config.customers;
        // recursive call to get the path to simplify the code - should not be infinite
        self.get_customer_id(address)
    }

    pub fn invalidate_api_key(&mut self) {
        self.api_key = update_api_key();
        confy::store_path("lexUploadConfig.yaml", self).expect("Failed to store new api key");
    }
}
