// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    agg_trx_producer::AggTranscriptProducer,
    dkg_manager::DKGManager,
    network::{IncomingRpcRequest, NetworkReceivers, NetworkSender},
    network_interface::DKGNetworkClient,
    DKGMessage,
};
use anyhow::{anyhow, Result};
use aptos_bounded_executor::BoundedExecutor;
use aptos_channels::{aptos_channel, message_queues::QueueStyle};
use aptos_config::config::{ReliableBroadcastConfig, SafetyRulesConfig};
use aptos_event_notifications::{
    EventNotification, EventNotificationListener, ReconfigNotification,
    ReconfigNotificationListener,
};
use aptos_logger::{debug, error, info, warn};
use aptos_network::{application::interface::NetworkClient, protocols::network::Event};
use aptos_reliable_broadcast::ReliableBroadcast;
use aptos_safety_rules::{safety_rules_manager::storage, PersistentSafetyStorage};
use aptos_types::{
    account_address::AccountAddress,
    dkg::{DKGStartEvent, DKGState, DefaultDKG, StartKeyGenEvent, RequestRevealEvent},
    epoch_state::EpochState,
    on_chain_config::{
        OnChainConfigPayload, OnChainConfigProvider, OnChainConsensusConfig,
        OnChainRandomnessConfig, RandomnessConfigMoveStruct, RandomnessConfigSeqNum, ValidatorSet,
    },
    validator_txn::{Topic, ValidatorTransaction},
};
use aptos_validator_transaction_pool::VTxnPoolState;
use futures::StreamExt;
use futures_channel::oneshot;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio_retry::strategy::ExponentialBackoff;

pub struct EpochManager<P: OnChainConfigProvider> {
    // Some useful metadata
    my_addr: AccountAddress,
    epoch_state: Option<Arc<EpochState>>,

    // Inbound events
    reconfig_events: ReconfigNotificationListener<P>,
    dkg_start_events: EventNotificationListener,

    // Msgs to DKG manager
    dkg_rpc_msg_tx:
        Option<aptos_channel::Sender<AccountAddress, (AccountAddress, IncomingRpcRequest)>>,
    dkg_manager_close_tx: Option<oneshot::Sender<oneshot::Sender<()>>>,
    dkg_start_event_tx: Option<aptos_channel::Sender<(), DKGStartEvent>>,
    vtxn_pool: VTxnPoolState,

    // Network utils
    self_sender: aptos_channels::Sender<Event<DKGMessage>>,
    network_sender: DKGNetworkClient<NetworkClient<DKGMessage>>,
    rb_config: ReliableBroadcastConfig,

    // Randomness overriding.
    randomness_override_seq_num: u64,

    key_storage: PersistentSafetyStorage,

    // Timelock DKG sessions
    // TODO: Implement in Phase 2 - Track active timelock DKG sessions per interval
    // Key: interval number, Value: DKGManager for that interval's key generation
    #[allow(dead_code)]
    timelock_dkg_managers: HashMap<u64, DKGManager<DefaultDKG>>,
}

impl<P: OnChainConfigProvider> EpochManager<P> {
    pub fn new(
        safety_rules_config: &SafetyRulesConfig,
        my_addr: AccountAddress,
        reconfig_events: ReconfigNotificationListener<P>,
        dkg_start_events: EventNotificationListener,
        self_sender: aptos_channels::Sender<Event<DKGMessage>>,
        network_sender: DKGNetworkClient<NetworkClient<DKGMessage>>,
        vtxn_pool: VTxnPoolState,
        rb_config: ReliableBroadcastConfig,
        randomness_override_seq_num: u64,
    ) -> Self {
        Self {
            my_addr,
            epoch_state: None,
            reconfig_events,
            dkg_start_events,
            dkg_rpc_msg_tx: None,
            dkg_manager_close_tx: None,
            self_sender,
            network_sender,
            vtxn_pool,
            dkg_start_event_tx: None,
            rb_config,
            randomness_override_seq_num,
            key_storage: storage(safety_rules_config),
            timelock_dkg_managers: HashMap::new(),
        }
    }

    fn process_rpc_request(
        &mut self,
        peer_id: AccountAddress,
        dkg_request: IncomingRpcRequest,
    ) -> Result<()> {
        if Some(dkg_request.msg.epoch()) == self.epoch_state.as_ref().map(|s| s.epoch) {
            // Forward to DKGManager if it is alive.
            if let Some(tx) = &self.dkg_rpc_msg_tx {
                let _ = tx.push(peer_id, (peer_id, dkg_request));
            }
        }
        Ok(())
    }

    fn on_dkg_start_notification(&mut self, notification: EventNotification) -> Result<()> {
        if let Some(tx) = self.dkg_start_event_tx.as_ref() {
            let EventNotification {
                subscribed_events, ..
            } = notification;
            for event in subscribed_events {
                if let Ok(dkg_start_event) = DKGStartEvent::try_from(&event) {
                    let _ = tx.push((), dkg_start_event);
                    return Ok(());
                } else if let Ok(timelock_start) = StartKeyGenEvent::try_from(&event) {
                    self.start_timelock_dkg(timelock_start);
                    return Ok(());
                } else if let Ok(timelock_reveal) = RequestRevealEvent::try_from(&event) {
                    self.process_timelock_reveal(timelock_reveal);
                    return Ok(());
                } else {
                    debug!("[DKG] on_dkg_start_notification: failed in converting a contract event to a dkg start event!");
                }
            }
        }
        Ok(())
    }

    pub async fn start(mut self, mut network_receivers: NetworkReceivers) {
        self.await_reconfig_notification().await;
        loop {
            let handling_result = tokio::select! {
                notification = self.dkg_start_events.select_next_some() => {
                    self.on_dkg_start_notification(notification)
                },
                reconfig_notification = self.reconfig_events.select_next_some() => {
                    self.on_new_epoch(reconfig_notification).await
                },
                (peer, rpc_request) = network_receivers.rpc_rx.select_next_some() => {
                    self.process_rpc_request(peer, rpc_request)
                },
            };

            if let Err(e) = handling_result {
                error!("{}", e);
            }
        }
    }

    async fn await_reconfig_notification(&mut self) {
        let reconfig_notification = self
            .reconfig_events
            .next()
            .await
            .expect("Reconfig sender dropped, unable to start new epoch");
        self.start_new_epoch(reconfig_notification.on_chain_configs)
            .await
            .unwrap();
    }

    async fn start_new_epoch(&mut self, payload: OnChainConfigPayload<P>) -> Result<()> {
        let validator_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");

        let epoch_state = Arc::new(EpochState::new(payload.epoch(), (&validator_set).into()));
        self.epoch_state = Some(epoch_state.clone());
        let my_index = epoch_state
            .verifier
            .address_to_validator_index()
            .get(&self.my_addr)
            .copied();

        let onchain_randomness_config_seq_num = payload
            .get::<RandomnessConfigSeqNum>()
            .unwrap_or_else(|_| RandomnessConfigSeqNum::default_if_missing());

        let randomness_config_move_struct = payload.get::<RandomnessConfigMoveStruct>();

        info!(
            epoch = epoch_state.epoch,
            local = self.randomness_override_seq_num,
            onchain = onchain_randomness_config_seq_num.seq_num,
            "Checking randomness config override."
        );
        if self.randomness_override_seq_num > onchain_randomness_config_seq_num.seq_num {
            warn!("Randomness will be force-disabled by local config!");
        }

        let onchain_randomness_config = OnChainRandomnessConfig::from_configs(
            self.randomness_override_seq_num,
            onchain_randomness_config_seq_num.seq_num,
            randomness_config_move_struct.ok(),
        );

        let onchain_consensus_config: anyhow::Result<OnChainConsensusConfig> = payload.get();
        if let Err(error) = &onchain_consensus_config {
            error!("Failed to read on-chain consensus config {}", error);
        }
        let consensus_config = onchain_consensus_config.unwrap_or_default();

        // Check both validator txn and randomness features are enabled
        let randomness_enabled =
            consensus_config.is_vtxn_enabled() && onchain_randomness_config.randomness_enabled();
        if let (true, Some(my_index)) = (randomness_enabled, my_index) {
            let DKGState {
                in_progress: in_progress_session,
                ..
            } = payload.get::<DKGState>().unwrap_or_default();

            let network_sender = self.create_network_sender();
            let rb = ReliableBroadcast::new(
                self.my_addr,
                epoch_state.verifier.get_ordered_account_addresses(),
                Arc::new(network_sender),
                ExponentialBackoff::from_millis(self.rb_config.backoff_policy_base_ms)
                    .factor(self.rb_config.backoff_policy_factor)
                    .max_delay(Duration::from_millis(
                        self.rb_config.backoff_policy_max_delay_ms,
                    )),
                aptos_time_service::TimeService::real(),
                Duration::from_millis(self.rb_config.rpc_timeout_ms),
                BoundedExecutor::new(8, tokio::runtime::Handle::current()),
            );
            let agg_trx_producer = AggTranscriptProducer::new(rb);

            let (dkg_start_event_tx, dkg_start_event_rx) =
                aptos_channel::new(QueueStyle::KLAST, 1, None);
            self.dkg_start_event_tx = Some(dkg_start_event_tx);

            let (dkg_rpc_msg_tx, dkg_rpc_msg_rx) = aptos_channel::new::<
                AccountAddress,
                (AccountAddress, IncomingRpcRequest),
            >(QueueStyle::FIFO, 100, None);
            self.dkg_rpc_msg_tx = Some(dkg_rpc_msg_tx);
            let (dkg_manager_close_tx, dkg_manager_close_rx) = oneshot::channel();
            self.dkg_manager_close_tx = Some(dkg_manager_close_tx);
            let my_pk = epoch_state
                .verifier
                .get_public_key(&self.my_addr)
                .ok_or_else(|| anyhow!("my pk not found in validator set"))?;
            let dealer_sk = self.key_storage.consensus_sk_by_pk(my_pk).map_err(|e| {
                anyhow!("dkg new epoch handling failed with consensus sk lookup err: {e}")
            })?;
            let dkg_manager = DKGManager::<DefaultDKG>::new(
                Arc::new(dealer_sk),
                my_index,
                self.my_addr,
                epoch_state,
                Arc::new(agg_trx_producer),
                self.vtxn_pool.clone(),
                false,
            );
            tokio::spawn(dkg_manager.run(
                in_progress_session,
                dkg_start_event_rx,
                dkg_rpc_msg_rx,
                dkg_manager_close_rx,
            ));
        };
        Ok(())
    }

    async fn on_new_epoch(&mut self, reconfig_notification: ReconfigNotification<P>) -> Result<()> {
        self.shutdown_current_processor().await;
        self.start_new_epoch(reconfig_notification.on_chain_configs)
            .await?;
        Ok(())
    }

    async fn shutdown_current_processor(&mut self) {
        if let Some(tx) = self.dkg_manager_close_tx.take() {
            let (ack_tx, ack_rx) = oneshot::channel();
            tx.send(ack_tx).unwrap();
            ack_rx.await.unwrap();
        }
    }

    fn create_network_sender(&self) -> NetworkSender {
        NetworkSender::new(
            self.my_addr,
            self.network_sender.clone(),
            self.self_sender.clone(),
        )
    }

    fn start_timelock_dkg(&mut self, event: StartKeyGenEvent) {
        info!("[Timelock] Starting keygen for interval {}", event.interval);

        // TODO: Implement actual DKG spawn in Phase 2
        // Required steps:
        // 1. Construct DKGSessionMetadata for this timelock interval
        //    - Use event.config.threshold and event.config.total_validators
        //    - Derive participants from current validator set
        //    - Set session_id based on interval number
        // 2. Spawn new DKGManager with is_timelock=true flag
        //    - Similar to randomness DKG but for timelock purposes
        //    - DKGManager will coordinate the distributed key generation
        // 3. Store DKGManager in self.timelock_dkg_managers[interval]
        // 4. When DKG completes, publish_public_key() will be called
        //    - Extract MPK (master public key) from transcript
        //    - Submit ValidatorTransaction::TimelockPublicKey
        // 5. Store secret share for later reveal
        //    - Call self.store_timelock_share(interval, share)

        warn!(
            "[Timelock] DKG spawn not yet implemented - stub only (interval {})",
            event.interval
        );
    }

    fn process_timelock_reveal(&self, event: RequestRevealEvent) {
        info!("[Timelock] Revealing share for interval {}", event.interval);

        // TODO: Implement actual share computation in Phase 4
        // Required steps:
        // 1. Retrieve secret share from persistent storage
        //    - Call self.retrieve_timelock_share(event.interval)
        //    - Share is a scalar in Fr (BLS12-381 scalar field)
        // 2. Compute BLS signature on interval identity
        //    - identity = format!("timelock-interval-{}", event.interval)
        //    - sig = H(identity)^secret_share  (where H: {0,1}* -> G1)
        //    - This is the IBE decryption key component
        // 3. Serialize signature to bytes (G1 point -> 48 bytes compressed)
        // 4. Create ValidatorTransaction::TimelockShare
        // 5. Submit to vtxn_pool with Topic::TIMELOCK
        // 6. On-chain aggregation will combine shares from threshold validators
        //    - Aggregated signature = sum of all validator signatures
        //    - This becomes the IBE decryption key for the interval

        // STUB: Submit dummy share for compilation
        let share = aptos_types::dkg::TimelockShare {
            interval: event.interval,
            share: vec![0u8; 48], // Dummy 48-byte G1 point (BLS12-381 compressed)
        };

        let txn = ValidatorTransaction::TimelockShare(share);
        // TODO: Uncomment when ready to actually submit
        // let _guard = self.vtxn_pool.put(Topic::TIMELOCK, Arc::new(txn), None);
        let _ = txn;

        warn!(
            "[Timelock] Share computation not yet implemented - submitted dummy share (interval {})",
            event.interval
        );
    }

    /// Store timelock secret share for later reveal.
    /// TODO: Implement in Phase 4 - Add persistent storage
    #[allow(dead_code)]
    fn store_timelock_share(&mut self, interval: u64, share: &[u8]) -> Result<()> {
        // TODO: Persist share to disk via key_storage or separate timelock storage
        // Format: Store as (interval -> secret_key_bytes) mapping
        // Security: Encrypt with validator's long-term key? Or rely on disk encryption?
        // Location: Extend PersistentSafetyStorage or create separate TimelockStorage?

        let _ = (interval, share);
        warn!("[Timelock] Share storage not implemented - data will be lost");
        Ok(())
    }

    /// Retrieve stored timelock secret share.
    /// TODO: Implement in Phase 4 - Add persistent storage
    #[allow(dead_code)]
    fn retrieve_timelock_share(&self, interval: u64) -> Result<Vec<u8>> {
        // TODO: Load share from disk
        // Return error if not found (validator may have joined after that interval)

        let _ = interval;
        Err(anyhow!("Share retrieval not implemented - no storage backend"))
    }
}
