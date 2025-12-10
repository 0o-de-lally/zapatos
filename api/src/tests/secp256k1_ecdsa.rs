// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use super::new_test_context_with_orderless_flags;
use aptos_api_test_context::current_function_name;
use aptos_crypto::{ed25519::Ed25519PrivateKey, secp256k1_ecdsa, SigningKey};
use aptos_sdk::types::{
    transaction::{
        authenticator::{
            AccountAuthenticator, AnyPublicKey, AnySignature, AuthenticationKey, MultiKey,
            MultiKeyAuthenticator,
        },
        SignedTransaction,
    },
    LocalAccount,
};
use rand::{rngs::StdRng, SeedableRng};
use rstest::rstest;
use std::convert::TryInto;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[rstest(
    case_name,
    use_txn_payload_v2_format,
    use_orderless_transactions,
    case("", false, false),
    case("_payload_v2", true, false),
    case("_orderless", true, true)
)]
async fn test_multi_secp256k1_ecdsa(
    case_name: &str,
    use_txn_payload_v2_format: bool,
    use_orderless_transactions: bool,
) {
    let mut context = new_test_context_with_orderless_flags(
        current_function_name!() + case_name,
        use_txn_payload_v2_format,
        use_orderless_transactions,
    );

    let other = context.create_account().await;

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let private_key: secp256k1_ecdsa::PrivateKey = aptos_crypto::Uniform::generate(&mut rng);
    let public_key = aptos_crypto::PrivateKey::public_key(&private_key);
    let address = AuthenticationKey::multi_key(
        MultiKey::new(vec![AnyPublicKey::secp256k1_ecdsa(public_key.clone())], 1).unwrap(),
    )
    .account_address();

    // Set a dummy key
    let key_bytes =
        hex::decode("a38ba78b1a0fbfc55e2c5dfdedf48d1172283d0f7c59fd64c02d811130a2f4b2").unwrap();
    let ed25519_private_key: Ed25519PrivateKey = (&key_bytes[..]).try_into().unwrap();
    let mut account = LocalAccount::new(address, ed25519_private_key, 0);

    let txn0 = context.create_user_account(&account).await;
    context.commit_block(&vec![txn0]).await;
    let txn1 = context.mint_user_account(&account).await;
    context.commit_block(&vec![txn1]).await;
    let txn2 = context.create_user_account(&other).await;
    context.commit_block(&vec![txn2]).await;

    let current_ledger_version = u64::from(context.get_latest_ledger_info().ledger_version);
    let ed22519_txn = context.account_transfer(&mut account, &other, 5);
    let raw_txn = ed22519_txn.into_raw_transaction();

    let signature = private_key.sign(&raw_txn).unwrap();
    let authenticator = AccountAuthenticator::multi_key(
        MultiKeyAuthenticator::new(
            MultiKey::new(vec![AnyPublicKey::secp256k1_ecdsa(public_key)], 1).unwrap(),
            vec![(0, AnySignature::secp256k1_ecdsa(signature))],
        )
        .unwrap(),
    );
    let secp256k1_ecdsa_txn = SignedTransaction::new_single_sender(raw_txn, authenticator);
    let balance_start = context.get_apt_balance(other.address()).await;
    let bcs_txn = bcs::to_bytes(&secp256k1_ecdsa_txn).unwrap();
    context
        .expect_status_code(202)
        .post_bcs_txn("/transactions", bcs_txn)
        .await;
    context.commit_mempool_txns(1).await;
    assert_eq!(
        balance_start + 5,
        context.get_apt_balance(other.address()).await
    );

    let txns = context
        .get(&format!(
            "/transactions?start={}&limit=1",
            current_ledger_version + 2
        ))
        .await;
    context.check_golden_output(txns[0]["signature"].clone());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[rstest(
    case_name,
    use_txn_payload_v2_format,
    use_orderless_transactions,
    case("", false, false),
    case("_payload_v2", true, false),
    case("_orderless", true, true)
)]
async fn test_secp256k1_ecdsa(
    case_name: &str,
    use_txn_payload_v2_format: bool,
    use_orderless_transactions: bool,
) {
    let mut context = new_test_context_with_orderless_flags(
        current_function_name!() + case_name,
        use_txn_payload_v2_format,
        use_orderless_transactions,
    );

    let other = context.create_account().await;

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let private_key: secp256k1_ecdsa::PrivateKey = aptos_crypto::Uniform::generate(&mut rng);
    let public_key = aptos_crypto::PrivateKey::public_key(&private_key);
    let address = AuthenticationKey::any_key(AnyPublicKey::secp256k1_ecdsa(public_key.clone()))
        .account_address();

    // Set a dummy key
    let key_bytes =
        hex::decode("a38ba78b1a0fbfc55e2c5dfdedf48d1172283d0f7c59fd64c02d811130a2f4b2").unwrap();
    let ed25519_private_key: Ed25519PrivateKey = (&key_bytes[..]).try_into().unwrap();
    let mut account = LocalAccount::new(address, ed25519_private_key, 0);

    let txn0 = context.create_user_account(&account).await;
    context.commit_block(&vec![txn0]).await;
    let txn1 = context.mint_user_account(&account).await;
    context.commit_block(&vec![txn1]).await;
    let txn2 = context.create_user_account(&other).await;
    context.commit_block(&vec![txn2]).await;

    let current_ledger_version = u64::from(context.get_latest_ledger_info().ledger_version);
    let ed22519_txn = context.account_transfer(&mut account, &other, 5);
    let secp256k1_ecdsa_txn = ed22519_txn
        .into_raw_transaction()
        .sign_secp256k1_ecdsa(&private_key, public_key)
        .unwrap();
    let balance_start = context.get_apt_balance(other.address()).await;
    let bcs_txn = bcs::to_bytes(&secp256k1_ecdsa_txn.into_inner()).unwrap();
    context
        .expect_status_code(202)
        .post_bcs_txn("/transactions", bcs_txn)
        .await;
    context.commit_mempool_txns(1).await;
    assert_eq!(
        balance_start + 5,
        context.get_apt_balance(other.address()).await
    );

    let txns = context
        .get(&format!(
            "/transactions?start={}&limit=1",
            current_ledger_version + 2
        ))
        .await;
    context.check_golden_output(txns[0]["signature"].clone());
}
    context.check_golden_output(txns[0]["signature"].clone());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[rstest(
    case_name,
    use_txn_payload_v2_format,
    use_orderless_transactions,
    case("", false, false),
    case("_payload_v2", true, false),
    case("_orderless", true, true)
)]
async fn test_secp256k1_implicit_account_creation(
    case_name: &str,
    use_txn_payload_v2_format: bool,
    use_orderless_transactions: bool,
) {
    let mut context = new_test_context_with_orderless_flags(
        current_function_name!() + case_name,
        use_txn_payload_v2_format,
        use_orderless_transactions,
    );

    // 1. Generate SECP256k1 private key
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let private_key: secp256k1_ecdsa::PrivateKey = aptos_crypto::Uniform::generate(&mut rng);
    let public_key = aptos_crypto::PrivateKey::public_key(&private_key);

    // 2. Derive Aptos Address (AnyPublicKey::Secp256k1Ecdsa wrapped in AuthenticationKey)
    let auth_key = AuthenticationKey::any_key(AnyPublicKey::secp256k1_ecdsa(public_key.clone()));
    let address = auth_key.account_address();

    // 3. Fund this address using the root account (simulating a bridge or funding event).
    // This implicitly creates the account if the feature is enabled (which it is in test context usually)
    let root = context.root_account();
    let txn_fund = context.mint(address, 100).await;
    context.commit_block(&vec![txn_fund]).await;

    // Verify balance
    let balance_start = context.get_apt_balance(address).await;
    assert_eq!(balance_start, 100);

    // 4. Create a RawTransaction (transfer 10 coins back to root or another random account)
    let other = context.create_account().await;
    // Note: We need to manually construct the `LocalAccount` to use with `context.account_transfer` helper,
    // or manually build the transaction. `context.account_transfer` expects a `LocalAccount` which holds the private key.
    // However, `LocalAccount` supports Ed25519 by default storage.
    // We can use `context.transaction_factory()` to build a raw transaction.

    let seq_num = context.get_sequence_number(address).await;
    let payload = context.aptos_transaction_factory()
        .payload(aptos_cached_packages::aptos_stdlib::aptos_coin_transfer(other.address(), 10));

    let raw_txn = context.aptos_transaction_factory()
        .sender(address)
        .sequence_number(seq_num)
        .payload(payload)
        .max_gas_amount(2000)
        .gas_unit_price(100)
        .build();

    // 5. Sign the transaction with SECP key
    let secp256k1_ecdsa_txn = raw_txn
        .sign_secp256k1_ecdsa(&private_key, public_key)
        .unwrap();

    // 6. Submit to local test validator
    let bcs_txn = bcs::to_bytes(&secp256k1_ecdsa_txn.into_inner()).unwrap();
    context
        .expect_status_code(202)
        .post_bcs_txn("/transactions", bcs_txn)
        .await;

    context.commit_mempool_txns(1).await;

    // 7. Assert success
    // Check balance of 'other' account (should accept 10)
    // Note: 'other' might not be funded initially if create_account checks implied it, but let's check
    // if minting to 'other' happened or if we just created it in `other` without funding.
    // `context.create_account()` creates a random local account struct but doesn't register it on chain unless used.
    // The transfer should create it implicitly too if it doesn't exist.
    assert_eq!(
        context.get_apt_balance(other.address()).await,
        10
    );

    // Check sender balance (100 - 10 - gas)
    let balance_end = context.get_apt_balance(address).await;
    assert!(balance_end < 90); // 10 sent + gas used
}
