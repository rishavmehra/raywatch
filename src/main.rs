use std::{fs, time::Duration, error::Error};
use actix_web::{ rt::time::sleep};
use chrono::Local;
use serde::{self, Deserialize};
use serde_json::Value;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient, 
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
    rpc_response::{RpcLogsResponse, Response}
};
use solana_sdk::commitment_config::CommitmentConfig;
use futures::{future::ok, StreamExt};


const LOG_LEVEL: &str = "LOG";
#[derive(Deserialize)]
struct EnvConfig{
    env: Config
}


#[derive(Deserialize)]
struct Config {
    MAINNET_RPC_URL: String,
    MAINNET_WSS_URL: String,
    RAYDIUM_LPV4: String,
    LOGS_INSTRUCTION: String
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>>{
    println!("hello world");

    let config = get_config();

    subscribe().await

    // println!("mainnet rpc url:{}", &config.env.MAINNET_RPC_URL);
    // println!("mainnet rpc url:{}", &config.env.MAINNET_WSS_URL);
    
}

async fn subscribe() -> Result<(), Box<dyn Error>> {
    let mut attempts =0;
    loop {
        attempts += 1;
        let ws_client_result = PubsubClient::new(&get_config().env.MAINNET_WSS_URL.as_str()).await;
    
        match ws_client_result {
            Ok(ws_client) =>{
                if attempts == 1 {
                    println!("Connected to Websocket.")
                } else {
                    println!("Connected ot Websocket after {attempts} attempts")
                }
                attempts = 0;
                let (mut stream, _) = ws_client
                    .logs_subscribe(
                        RpcTransactionLogsFilter::Mentions(vec![get_config().env.RAYDIUM_LPV4.to_string()]),
                        RpcTransactionLogsConfig {
                            commitment: Some(CommitmentConfig::finalized()),
                        },
                    )
                    .await?;

                println!("Subscribed to Raydium Liquidity pool");

                loop {
                    match stream.next().await {
                        Some(res) =>{
                            // todo: need to add the message parsing 
                            println!("Received log: {:?}", res);
                        }
                        None =>{
                            println!("Steam ended");
                            break;
                        }
                    }
                }
            }

            Err(e)=>{
                eprintln!("Failed to connect  to websocke. Attempts{attempts} of 10. Error: {e}");
                if attempts >= 10 {
                    eprintln!("Max retry limit reached");
                    return  Err("Max limit reached out!".into());
                }
                sleep(Duration::from_secs(5)).await;
            }
        }
    
    }

}

// read the env's from the config file(config.toml)
fn get_config() -> EnvConfig {
    let config_file = fs::read_to_string("config.toml").expect("config.toml is not specified");
    let config = toml::from_str(&config_file).expect("Env are not spcified");
    config
}


async fn process_message(res: Response<RpcLogsResponse>) {
    let value = res.value;
    for log in value.logs {
        if !log.contains(get_config().env.LOGS_INSTRUCTION.as_str()){
            continue;
        }
        let signature_str = &value.signature;
        get_tokens()
    }
}

// TODO: add the token filters

// async fn get_tokens(sign: &str, program:String){
//     let result = token_transactions
// }



// pub async fn token_transactions(sign: &str, encode: &str, http: &str)-> Result<Value, Box<dyn Error>>{

//     let logger = Logger::new(format)


// }

struct Logger {
    prefix: String,
    date_format: String,
}

impl Logger {
    fn new(prefix: String) -> Self{
        Logger{
            prefix,
            date_format: String::from("%Y-%m-%d %H:%M:%S"),
        }
    }

    fn log(&self, message: String) -> String {
        let log = format!("{} {}", self.prefix_with_date(), message);
        println!("{}", log);
        log
    }

    fn debug(&self, message: String) -> String{
        let log = format!("{} [{}] {}", self.prefix_with_date(), "DEBUG", message);
        if LogLevel::new().is_debug(){
            println!("{}", log)
        }
        log
    }

    fn error(&self, message: String) -> String {
        let log = format!("{} [{}] {}", self.prefix_with_date(), "ERROR", message);
        log
    }

    fn prefix_with_date(&self) -> String {
        let date = Local::now();
        format!("[{}] {}", date.format(self.date_format.as_str()), self.prefix)
    }
}

struct LogLevel<'a>{
    level: &'a str
}

impl LogLevel<'_>{
    fn new() -> Self{
        let level = LOG_LEVEL;
        LogLevel { level }
    }
    fn is_debug(&self) -> bool {
        self.level.to_lowercase().eq("debug")
    }
}