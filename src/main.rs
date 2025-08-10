use std::{fs, time::Duration, error::Error};
use actix_web::{dev::Response, rt::time::sleep};
use serde::{self, Deserialize};
use solana_client::{
    nonblocking::pubsub_client::PubsubClient, 
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter}
};
use solana_sdk::commitment_config::CommitmentConfig;
use futures::StreamExt;


#[derive(Deserialize)]
struct EnvConfig{
    env: Config
}

#[derive(Deserialize)]
struct Config {
    MAINNET_RPC_URL: String,
    MAINNET_WSS_URL: String,
    RAYDIUM_LPV4: String
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
                let (mut launch, _) = ws_client
                    .logs_subscribe(
                        RpcTransactionLogsFilter::Mentions(vec![get_config().env.RAYDIUM_LPV4.to_string()]),
                        RpcTransactionLogsConfig {
                            commitment: Some(CommitmentConfig::finalized()),
                        },
                    )
                    .await?;

                println!("Subscribed to Raydium Liquidity pool");

                loop {
                    match launch.next().await {
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





