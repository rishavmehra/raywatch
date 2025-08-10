use std::fs;
use serde::{self, Deserialize};

#[derive(Deserialize)]
struct EnvConfig{
    env: Config
}

#[derive(Deserialize)]
struct Config {
    MAINNET_RPC_URL: String,
    MAINNET_WSS_URL: String
}

fn main(){
    println!("hello world");

    let config = get_config();

    // println!("mainnet rpc url:{}", &config.env.MAINNET_RPC_URL);
    // println!("mainnet rpc url:{}", &config.env.MAINNET_WSS_URL);
    
}

async fn subscribe()-> Result<()>{


}

// read the env's from the config file(config.toml)
fn get_config() -> EnvConfig {
    let config_file = fs::read_to_string("config.toml").expect("config.toml is not specified");
    let config = toml::from_str(&config_file).expect("Env are not spcified");
    config
}

