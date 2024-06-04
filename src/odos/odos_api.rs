use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, vec};
use serde_json::Value;


#[allow(non_camel_case_types, non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct InputTokens {
    tokenAddress: String,
    amount: String,
}

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct OutputTokens {
    tokenAddress: String,
    proportion: u8,
}

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct QuoteData {
    chainId: u128,
    inputTokens: Vec<InputTokens>,
    outputTokens: Vec<OutputTokens>,
    slippageLimitPercent: u8,
    userAddr: String,
    referralCode: u128,
    compact: bool,
}

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct AssembleData {
    userAddr: String,
    pathId: String,
    simulate: bool,
}


pub async fn assemble(wallet_address: String, path_id: String) -> Result<HashMap<String, Value>, Box<dyn Error>> {
    let assemble_data = AssembleData{
        userAddr: wallet_address,
        pathId: path_id,
        simulate: false,
    };

    let client = reqwest::Client::new();

    let response = client.post("https://api.odos.xyz/sor/assemble")
    .header("Content-Type", "application/json")
    .json(&assemble_data)
    .send()
    .await?;

    let status_code = &response.status();

    if status_code.as_u16() != 200 {
        panic!("Status code != 200, msg: {:?}", response.text().await?)
    }

    let json_response = response.json::<HashMap<String, Value>>().await?;

    Ok(json_response)
}

pub async fn get_quote(wallet_address: String, chain_id: u128, path: Vec<String>, amount: u128) -> Result<HashMap<String, Value>, Box<dyn Error>> {
    let input_tokens = vec![
        InputTokens{
            tokenAddress: path[0].to_string(), amount: amount.to_string(),
        }
    ];

    let output_tokens = vec![
            OutputTokens{
            tokenAddress: path[1].to_string(), proportion: 1,
        }
    ];

    let quote_request = QuoteData {
        chainId: chain_id,
        inputTokens: input_tokens,
        outputTokens: output_tokens,
        slippageLimitPercent: 1,
        userAddr: wallet_address,
        referralCode: 0,
        compact: true,
    };

    let client = reqwest::Client::new();

    let response = client.post("https://api.odos.xyz/sor/quote/v2")
    .header("Content-Type", "application/json")
    .json(&quote_request)
    .send()
    .await?;

    let status_code = &response.status();

    if status_code.as_u16() != 200 {
        panic!("Status code != 200")
    }

    let json_response = response.json::<HashMap<String, Value>>().await?;

    Ok(json_response)
}