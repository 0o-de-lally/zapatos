use aptos_crypto::{secp256k1_ecdsa, Uniform};
use aptos_forge::{Swarm, SwarmExt};
use aptos_logger::info;
use aptos_sdk::types::transaction::authenticator::{AnyPublicKey, AuthenticationKey};
use rand::{rngs::StdRng, SeedableRng};

#[tokio::test]
async fn test_secp_transactions() {
    // Correctly initialize swarm using the smoke_test helper
    let mut swarm = crate::smoke_test_environment::new_local_swarm_with_aptos(1).await;
    
    // SwarmExt trait is needed for this method
    let info = swarm.aptos_public_info();

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let private_key: secp256k1_ecdsa::PrivateKey = Uniform::generate(&mut rng);
    let public_key = aptos_crypto::PrivateKey::public_key(&private_key);

    let auth_key = AuthenticationKey::any_key(AnyPublicKey::secp256k1_ecdsa(public_key.clone()));
    let account_address = auth_key.account_address();

    info!("Generated SECP Account Address: {}", account_address);

    // Fund the account
    info.mint(account_address, 1_000_000).await.unwrap();
    info!("Account funded");

    let root_account = info.root_account();
    let transfer_amount = 100;

    // Create transaction payload
    let payload = aptos_cached_packages::aptos_stdlib::aptos_coin_transfer(root_account.address(), transfer_amount);

    let transaction_factory = info.transaction_factory();
    let raw_txn = transaction_factory
        .sender(account_address)
        .sequence_number(0) // New account, seq num 0
        .payload(payload)
        .max_gas_amount(5_000)
        .gas_unit_price(100)
        .build();

    // Sign with SECP key
    let signed_txn = raw_txn
        .sign_secp256k1_ecdsa(&private_key, public_key)
        .unwrap();

    // Submit transaction
    info.client().submit_and_wait(&signed_txn.into_inner()).await.unwrap();
    info!("Transaction submitted and verified!");

    // Check balance of sender
    let balance = info.client().get_account_balance(account_address).await.unwrap();
    info!("Final balance: {}", balance.inner());

    // 1_000_000 - 100 - gas
    assert!(balance.into_inner() < 1_000_000 - 100);
}
