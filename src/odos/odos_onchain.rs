use std::{collections::HashMap, error::Error, sync::Arc};
use ethers::prelude::*;
use serde_json::Value;
use super::odos_api;


const ZEROADDRESS: &str = "0x0000000000000000000000000000000000000000";
const AVAILABLE_CHAIN_ID: &[u128; 12] = &[
    1, // Ethereum
    42161, // Arbitrum
    10, // Optimism
    56, // Bsc
    137, // Polygon
    250, // Fantom
    324, // ZkSync Era
    5000, // Mantle
    8453, // Base
    34443, // Mode
    43114, // Avalanche
    59144, // Linea
];

abigen!(Erc20Contract, "src/abi/erc20.json");

pub async fn swap(rpc: String, private_key: String, amount: u128, path: Vec<String>) -> Result<(), Box<dyn Error>> {
    let provider = Arc::new(
        {
                let provider = Provider::<Http>::try_from(
                    rpc,
                ).unwrap();
        
                let chain_id = provider.get_chainid().await?;

                if AVAILABLE_CHAIN_ID.contains(&chain_id.as_u128()) == false {
                    panic!("Odos does not support {:?} chain id | Available: {:#?}", &chain_id, &AVAILABLE_CHAIN_ID)
                }

                let wallet = private_key
                    .parse::<LocalWallet>()?
                    .with_chain_id(chain_id.as_u64());

                SignerMiddleware::new(provider, wallet)
            }
    );

    let address_string = format!("{:?}", provider.address());

    let quote = odos_api::get_quote(
        address_string.clone(), 
        provider.get_chainid().await?.as_u128(), 
        path.clone(), 
        amount.clone(),
    ).await?;

    println!("Quote result: {:?}\n", quote);

    let path_id = quote
    .get("pathId")
    .unwrap()
    .as_str()
    .unwrap()
    .trim()
    .to_string();

    let assemble_data: HashMap<String, Value> = odos_api::assemble(
        address_string, 
        path_id,
    ).await?;

    println!("Assemble data: {:?}\n", &assemble_data);

    let transaction = assemble_data.get("transaction").unwrap().as_object().unwrap();
    let chain_id = transaction.get("chainId").unwrap().as_u64().unwrap();
    let to = transaction.get("to").unwrap().as_str().unwrap();
    let gas = transaction.get("gas").unwrap().as_u64().unwrap();
    let gas_price = transaction.get("gasPrice").unwrap().as_u64().unwrap();
    let data = transaction.get("data").unwrap().as_str().unwrap();
    let mut nonce = transaction.get("nonce").unwrap().as_u64().unwrap();

    if path[0].to_string() != ZEROADDRESS.to_string() {
        let erc20_address = path[0].clone().parse::<Address>()?;
        let spender = to.parse::<Address>()?;

        let contract = Erc20Contract::new(
            erc20_address, 
            provider.clone(),
        );

        let allowance_call = contract.allowance(
            provider.address(),
            spender.clone(),
        );

        let allowance: U256 = allowance_call.call().await?;

        println!("Allowance: {:?}", allowance);

        if &allowance.as_u128() < &amount {
            let approve = contract.approve(
                spender, 
                U256::from(amount.clone()),
            );

            let pending_approve = approve.send().await?;
        
            let sended_tx: Option<TransactionReceipt> = pending_approve.await?;
            let unwraped_tx: TransactionReceipt = sended_tx.unwrap();

            println!("Approve transaction hash: {:?} | Status: {:?}", unwraped_tx.transaction_hash, unwraped_tx.status.unwrap());

            if unwraped_tx.status.unwrap() != U64::from(1) {
                panic!("Approve transaction failed")
            }

            nonce += 1;

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    let transaction = TransactionRequest {
        from: Some(provider.address()),
        to: Some(to.parse()?),
        gas: Some(U256::from(gas)),
        gas_price: Some(U256::from(gas_price)),
        value: Some(U256::from(amount)),
        data: Some(data.parse()?),
        nonce: Some(U256::from(nonce)),
        chain_id: Some(chain_id.into()),
        ..Default::default()
    };

    let pending_tx = provider.send_transaction(
        transaction, 
        None
    ).await?;

    let sended_tx: Option<TransactionReceipt> = pending_tx.await?;
    let unwraped_tx: TransactionReceipt = sended_tx.unwrap();

    println!("Swap transaction hash: {:?} | Status: {:?}", unwraped_tx.transaction_hash, unwraped_tx.status.unwrap());

    if unwraped_tx.status.unwrap() != U64::from(1) {
        panic!("Swap transaction failed")
    }

    Ok(())
}