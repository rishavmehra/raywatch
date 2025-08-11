use std::{any::{Any, TypeId}, error::Error, fmt::format, fs, time::Duration};
use actix_web::{ rt::time::sleep};
use async_recursion::async_recursion;
use reqwest::{header, Client};
use chrono::Local;
use serde::{self, Deserialize};
use serde_json::Value;
use solana_client::{
 nonblocking::pubsub_client::PubsubClient, rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter}, rpc_response::{Response, RpcLogsResponse}
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
    let logger = Logger::new("Setup".to_string());

    logger.log(format!("Solana WSS: {:?}", get_config().env.MAINNET_WSS_URL.as_str()));
    logger.log(format!("Solan http: {:?}", get_config().env.MAINNET_RPC_URL.as_str()));
    logger.log(format!("Log instructtion: {:?}", get_config().env.LOGS_INSTRUCTION.as_str()));

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
                            process_message(res).await;
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
        let sign_str = &value.signature;
        get_tokens(&sign_str, get_config().env.RAYDIUM_LPV4.to_string()).await;
    }
}

async fn get_tokens(sign: &str, pid:String){
    println!("reached1");
    let result = token_transactions(sign, "jsonParsed", &get_config().env.MAINNET_RPC_URL.as_str()).await.expect("failed to get the data");
    let ix = get_ix_by_pid(result, pid);
    
    token_info(ix);
}

#[async_recursion]
pub async fn token_transactions(sign: &str, encode: &str, http: &str)-> Result<Value, Box<dyn Error>>{
    let logger = Logger::new(String::from("Token handler"));
    let client = Client::new();
        let mut headers = header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let client = Client::new();
        let json_data = format!(
            "
        {{
            \"jsonrpc\": \"2.0\",
            \"id\": 1,
            \"method\": \"getTransaction\",
            \"params\": [
                \"{sign}\",
                {{
                    \"encoding\": \"{encode}\",
                    \"maxSupportedTransactionVersion\": 0
                }}
            ]
        }}"
        );
        let response = client.post(http).headers(headers).body(json_data).send().await?;

        let body= response.text().await?;

        let mut body_json: Value = serde_json::from_str(body.as_str()).expect("Failed to parse JSON");
        logger.debug(format!("getTransaction [\"result\"] type: {:?}, is type string: {:?}", body_json["result"].type_id(), body_json["result"].type_id()==TypeId::of::<String>()));

        if body_json["result"].type_id() == TypeId::of::<String>() {
            logger.debug(format!("Resending getTransaction request for \"{}\" signature", sign));
            sleep(Duration::from_secs(1)).await;
            body_json = token_transactions(sign, encode, http).await.unwrap();
        }
        return Ok(body_json);
}


fn get_ix(json: Value) -> Vec<serde_json::Value>{
    let mut ix = Vec::new();
    if let Some(simple_ixs) = json["result"]["transaction"]["message"]["instructions"].as_array()
    {
        for simple_ix in simple_ixs {
            ix.push(simple_ix.clone());
        }
    }
    ix
}

fn token_info(ixs: Vec<serde_json::Value>){
    let logger = Logger::new(String::from("token handler"));
    for ix in ixs {
        let tokens = get_token_info(ix);
        let token: &str;
        if "So11111111111111111111111111111111111111112" == tokens.0.as_str().unwrap(){
            token = &tokens.1.as_str().unwrap();
        } else {
            token = &tokens.2.as_str().unwrap();
        }
        let lp_pair = &tokens.2.as_str().unwrap();
        logger.log(format!("new pair found(Token: {} LP Pair: {})", token, lp_pair));
    }
}

fn get_token_info(ix: serde_json::Value) -> (serde_json::Value, serde_json::Value, serde_json::Value){
    let accounts = &ix["accounts"];
    let pair = &accounts[4];
    let token0 = &accounts[8];
    let token1 = &accounts[9];
    (token0.clone(), token1.clone(), pair.clone())
}


fn get_ix_by_pid(json: Value, pid: String) -> Vec<serde_json::Value>{
    let mut filtred_ix = Vec::new();
    let ixs = get_ix(json);
    if ixs.is_empty(){
        let logger = Logger::new(String::from("Token handler"));
        logger.error("nothing found in instrauction".to_string());
        return filtred_ix;
    }
    for ix in ixs {
        if ix["programId"].eq(&pid){
            filtred_ix.push(ix);
        }
    }
    filtred_ix
}



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