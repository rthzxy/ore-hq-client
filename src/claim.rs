use std::time::Duration;

use clap::Parser;
use solana_sdk::{signature::Keypair, signer::Signer};


#[derive(Debug, Parser)]
pub struct ClaimArgs {
    #[arg(
        long,
        value_name = "AMOUNT",
        default_value = "0.0",
        help = "Amount of ore to claim."
    )]
    pub amount: f64,
}

pub async fn claim(args: ClaimArgs, key: Keypair, url: String, unsecure: bool) {
    let claim_amount = (args.amount * 10f64.powf(ore_api::consts::TOKEN_DECIMALS as f64)) as u64;

    let base_url = url;
    let client = reqwest::Client::new();

    let url_prefix = if unsecure {
        "http".to_string()
    } else {
        "https".to_string()
    };


    loop {
        let balance = client.get(format!("{}://{}/miner/rewards?pubkey={}", url_prefix, base_url, key.pubkey().to_string())).send().await.unwrap().text().await.unwrap();
        println!("Claimable Rewards: {}", balance);
        let claim_amount = if claim_amount != 0 {
            claim_amount
        } else {
            let balance_grains = (balance.parse::<f64>().unwrap() * 10f64.powf(ore_api::consts::TOKEN_DECIMALS as f64)) as u64;
            balance_grains
        };
        println!("Sending claim request for amount {}...", claim_amount);
        let resp = client.post(format!("{}://{}/claim?pubkey={}&amount={}", url_prefix, base_url, key.pubkey().to_string(), claim_amount)).send().await;

        match resp {
            Ok(res) => {
                match res.text().await.unwrap().as_str() {
                    "SUCCESS" => {
                        println!("Successfully claimed rewards!");
                        break;
                    },
                    _ => {
                        println!("Retrying in 10 seconds...");
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                }

            },
            Err(_e) => {
                println!("Retrying in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
