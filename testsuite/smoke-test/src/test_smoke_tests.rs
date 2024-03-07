// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SwarmBuilder,
    test_utils::{MAX_CONNECTIVITY_WAIT_SECS, MAX_HEALTHY_WAIT_SECS},
};
use aptos_config::config::{NodeConfig, OverrideNodeConfig};
use aptos_forge::{NodeExt, Swarm};
use aptos_move_debugger::aptos_debugger::AptosDebugger;
use aptos_types::{account_config, account_address::AccountAddress};
use aptos_vm::move_vm_ext::SessionExt;
use move_core_types::{
    identifier::Identifier,
    language_storage::ModuleId,
    value::{serialize_values, MoveValue},
};
use move_vm_types::gas::UnmeteredGasMeter;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Bring up a swarm normally, then run get_bin, and bring up a VFN.
/// Previously get_bin triggered a rebuild of aptos-node, which caused issues that were only seen
/// during parallel execution of tests.
/// This test should make regressions obvious.
#[tokio::test]
async fn test_aptos_node_after_get_bin() {
    let mut swarm = SwarmBuilder::new_local(1)
        .with_aptos()
        .with_init_config(Arc::new(|_, conf, _| {
            conf.api.failpoints_enabled = true;
        }))
        .build()
        .await;
    let version = swarm.versions().max().unwrap();
    let validator_peer_ids = swarm.validators().map(|v| v.peer_id()).collect::<Vec<_>>();

    // Before #5308 this re-compiled aptos-node and caused a panic on the vfn.
    let _aptos_cli = crate::workspace_builder::get_bin("aptos");

    let validator = validator_peer_ids[0];
    let _vfn = swarm
        .add_validator_fullnode(
            &version,
            OverrideNodeConfig::new_with_default_base(NodeConfig::get_default_vfn_config()),
            validator,
        )
        .unwrap();

    for fullnode in swarm.full_nodes_mut() {
        fullnode
            .wait_until_healthy(Instant::now() + Duration::from_secs(MAX_HEALTHY_WAIT_SECS))
            .await
            .unwrap();
        fullnode
            .wait_for_connectivity(Instant::now() + Duration::from_secs(MAX_CONNECTIVITY_WAIT_SECS))
            .await
            .unwrap();
    }
}

// reproduce the FAILED_TO_DESERIALIZE_RESOURCE when using debugger.
// prevents running writeset transactions that use `exists<T>(addr);`
#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn repro_deserialize_error_in_debugger() {
    let mut swarm = SwarmBuilder::new_local(1)
        .with_aptos()
        .with_init_config(Arc::new(|_, conf, _| {
            conf.api.failpoints_enabled = true;
        }))
        .build()
        .await;
    let val = swarm.validators_mut().nth(0).unwrap();
    let db_path = &val.config().storage.dir();
    val.stop();

    let debug = AptosDebugger::db(db_path).expect("debugger");
    let version = debug.get_latest_version().await.expect("version");
    let rando = AccountAddress::random();
    let _ = debug
        .run_session_at_version(version, |session| {
            /////// all account creation fails //////
            execute_fn(session, "account", "create_account_unchecked", vec![&MoveValue::Address(rando)]);

            /////// any attempt to add new user state to offline db is not possible //////
            execute_fn(session, "repro_deserialize", "should_init_struct", vec![&MoveValue::Signer(rando)]);

            /////// creating a signer will also fail //////
            execute_fn(session, "create_signer", "create_signer", vec![&MoveValue::Signer(rando)]);

            Ok(())
        })
        .expect("could run session");
}

fn execute_fn(session: &mut SessionExt, module: &str, function: &str, args: Vec<&MoveValue>) {
    let r = session
        .execute_function_bypass_visibility(
            &ModuleId::new(
                account_config::CORE_CODE_ADDRESS,
                Identifier::new(module).unwrap(),
            ),
            &Identifier::new(function).unwrap(),
            vec![],
            serialize_values(args),
            &mut UnmeteredGasMeter,
        )
        .expect("run function");
    dbg!(&r);
}
