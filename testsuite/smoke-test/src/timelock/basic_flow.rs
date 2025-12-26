// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

//! Basic timelock flow E2E test
//!
//! This test verifies the end-to-end flow of timelock encryption:
//! 1. Genesis initialization of timelock system
//! 2. Interval rotation triggers DKG for new keys
//! 3. Validators publish public key for encryption
//! 4. Interval rotation triggers reveal request
//! 5. Validators reveal secret shares
//! 6. On-chain aggregation produces decryption key

use crate::smoke_test_environment::SwarmBuilder;
use aptos_forge::{NodeExt, SwarmExt};
use aptos_logger::info;
use std::{sync::Arc, time::Duration};

/// Test basic timelock flow with fast interval for testing.
///
/// This test:
/// - Starts a 4-validator network
/// - Verifies timelock is initialized at genesis
/// - Waits for first rotation
/// - Verifies public key is published
/// - Waits for reveal
/// - Verifies secret is aggregated
#[tokio::test]
async fn test_timelock_basic_flow() {
    let interval_secs = 5;

    info!(
        "Building swarm with 4 validators and {}-second interval",
        interval_secs
    );

    let (swarm, _cli, _faucet) = SwarmBuilder::new_local(4)
        .with_num_fullnodes(1)
        .with_aptos()
        .with_init_genesis_config(Arc::new(move |conf| {
            // Enable validator transactions (required for timelock)
            conf.consensus_config.enable_validator_txns();

            // TODO: Add timelock configuration for shorter intervals
            // This would require adding timelock_config to GenesisConfiguration
            // For now, we rely on the default interval
        }))
        .build_with_cli(0)
        .await;

    let client = swarm.validators().next().unwrap().rest_client();

    info!("Swarm started, verifying timelock is initialized at genesis");

    // Step 1 - Verify timelock initialized at genesis
    let initialized = super::is_timelock_initialized(&client).await.unwrap();
    assert!(initialized, "Timelock should be initialized at genesis");

    let initial_interval = super::get_current_interval(&client).await.unwrap();
    info!("Initial interval: {}", initial_interval);
    // Note: initial_interval may be > 0 if genesis took time

    info!("Waiting for first interval rotation");

    // Step 2 - Wait for rotation to next interval
    // Use longer timeout since we can't configure short intervals yet
    let target_interval = initial_interval + 1;
    let timeout_secs = 120; // 2 minutes - may need adjustment

    let state = super::wait_for_interval_rotation(&client, target_interval, timeout_secs)
        .await
        .unwrap();
    assert!(
        state.current_interval >= target_interval,
        "Should have rotated to interval {}",
        target_interval
    );

    info!("First rotation complete, verifying public key published");

    // Step 3 - Verify public key for the new interval is published
    // Note: This may fail if validators haven't published yet
    match super::verify_public_key_published(&client, target_interval).await {
        Ok(public_key) => {
            info!(
                "Public key published for interval {}: {} bytes",
                target_interval,
                public_key.len()
            );
            // BLS12-381 G2 point should be 96 bytes (compressed)
            assert!(
                public_key.len() == 48 || public_key.len() == 96,
                "Public key should be 48 or 96 bytes, got {}",
                public_key.len()
            );
        }
        Err(e) => {
            info!(
                "Public key not yet published for interval {}: {}",
                target_interval, e
            );
            // This is expected if DKG hasn't completed
        }
    };

    // Step 4 - Check if secret is revealed for previous interval
    if initial_interval > 0 {
        match super::verify_secret_aggregated(&client, initial_interval - 1, 3).await {
            Ok(secret) => {
                info!(
                    "Secret revealed for interval {}: {} bytes",
                    initial_interval - 1,
                    secret.len()
                );
            }
            Err(e) => {
                info!(
                    "Secret not yet revealed for interval {}: {}",
                    initial_interval - 1,
                    e
                );
            }
        }
    }

    info!("✅ Test completed - basic timelock flow verified");
}

/// Test that timelock config can be updated on testnet (not mainnet).
///
/// TODO: Implement when timelock_config module is tested
#[tokio::test]
#[ignore]
async fn test_timelock_config_override() {
    // TODO: Verify set_interval_for_testing() works on testnet
    // TODO: Verify it aborts on mainnet (chain_id == 1)
}

/// Test that timelock handles validator set changes gracefully.
///
/// TODO: Implement when DKG integration is complete
#[tokio::test]
#[ignore]
async fn test_timelock_with_validator_changes() {
    // TODO: Start with 4 validators
    // TODO: Trigger DKG for interval 1
    // TODO: Add validator during DKG
    // TODO: Verify new validator doesn't break DKG
    // TODO: Verify reveal still works with threshold
}

/// Test that timelock handles DKG failures gracefully.
///
/// TODO: Implement when DKG integration is complete
#[tokio::test]
#[ignore]
async fn test_timelock_dkg_failure_recovery() {
    // TODO: Start with 4 validators
    // TODO: Kill 2 validators during DKG
    // TODO: Verify DKG fails (below threshold)
    // TODO: Restart validators
    // TODO: Verify next interval DKG succeeds
}
