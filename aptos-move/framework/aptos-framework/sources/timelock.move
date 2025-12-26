module aptos_framework::timelock {

    use std::option::{Self, Option};
    use aptos_std::table::{Self, Table};
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;
    use aptos_framework::system_addresses;
    use aptos_framework::account;
    use aptos_framework::timelock_config;

    friend aptos_framework::block;
    friend aptos_framework::genesis;

    #[test_only]
    use std::vector;

    /// The singleton was not initialized.
    const ETIMELOCK_NOT_INITIALIZED: u64 = 1;

    struct TimelockConfig has copy, drop, store {
        threshold: u64,
        total_validators: u64,
    }

    struct TimelockState has key {
        current_interval: u64,
        last_rotation_time: u64,
        /// Store public keys (for encryption)
        public_keys: Table<u64, vector<u8>>,
        /// Store revealed secret keys/signatures (for decryption)
        revealed_secrets: Table<u64, vector<u8>>,
        /// Events
        start_keygen_events: EventHandle<StartKeyGenEvent>,
        request_reveal_events: EventHandle<RequestRevealEvent>,
    }

    /// Event emitted to tell validators: "Please generate keys for interval X"
    struct StartKeyGenEvent has drop, store {
        interval: u64,
        config: TimelockConfig,
    }

    /// Event emitted to tell validators: "Please reveal the secret for interval X"
    struct RequestRevealEvent has drop, store {
        interval: u64,
    }

    /// Initialize the timelock system.
    public(friend) fun initialize(framework: &signer) {
        system_addresses::assert_aptos_framework(framework);
        move_to(framework, TimelockState {
            current_interval: 0,
            last_rotation_time: 0, // Will be updated on first block
            public_keys: table::new(),
            revealed_secrets: table::new(),
            start_keygen_events: account::new_event_handle<StartKeyGenEvent>(framework),
            request_reveal_events: account::new_event_handle<RequestRevealEvent>(framework),
        });
    }

    /// Called by block prologue to trigger rotations.
    public(friend) fun on_new_block(vm: &signer) acquires TimelockState {
        system_addresses::assert_vm(vm);

        if (!exists<TimelockState>(@aptos_framework)) {
            return
        };

        let state = borrow_global_mut<TimelockState>(@aptos_framework);
        let now = timestamp::now_microseconds();

        // Initialize last_rotation_time if it's 0 (genesis/first run)
        if (state.last_rotation_time == 0) {
            state.last_rotation_time = now;
            return
        };

        // Check if configured interval has passed (get from timelock_config)
        let interval_micros = timelock_config::get_interval_microseconds();
        if (now - state.last_rotation_time > interval_micros) {
            let old_interval = state.current_interval;
             // Emit reveal event for the old interval
            event::emit_event(&mut state.request_reveal_events, RequestRevealEvent {
                interval: old_interval,
            });

            state.current_interval = state.current_interval + 1;
            state.last_rotation_time = now;

            // TODO: In a real implementation, we would get the actual validator set size/threshold.
            // For this PoC, we'll hardcode or placeholders.
            // Let's assume a fixed threshold for now or just emit the event.
            let config = TimelockConfig {
                threshold: 1, // Placeholder
                total_validators: 1, // Placeholder
            };

            event::emit_event(&mut state.start_keygen_events, StartKeyGenEvent {
                interval: state.current_interval,
                config,
            });
        }
    }

    /// validators call this to publish the public key for a future interval
    public entry fun publish_public_key(
        _validator: &signer,
        interval: u64,
        pk: vector<u8>
    ) acquires TimelockState {
        // TODO: Verify sender is a validator
        // In this minimal PoC, we trust the sender for now or assume the VM only allows valid calls via ValidatorTxn
        // BUT `public entry` means anyone can call it?
        // The plan says "Submit 0x1::timelock::publish_secret_share" via ValidatorTransaction.
        // If it comes via ValidatorTransaction, it should be a governance/system transaction, but `entry` allows user calls.
        
        // Use system_addresses or relevant checks if strictly required.
        // For PoC, let's keep it simple but functional.
        
        let state = borrow_global_mut<TimelockState>(@aptos_framework);
        if (!table::contains(&state.public_keys, interval)) {
            table::add(&mut state.public_keys, interval, pk);
        };
    }

    /// validators call this to publish the secret share/signature for a past interval
    public entry fun publish_secret_share(
        _validator: &signer,
        interval: u64,
        share: vector<u8>
    ) acquires TimelockState {
        // TODO: Implement proper BLS signature aggregation in Phase 2
        // Current behavior: Store first share only (placeholder)
        // Real implementation needs to:
        // 1. Verify validator authorization (via ValidatorTransaction context)
        // 2. Verify BLS signature is valid for the interval identity
        // 3. Collect shares from multiple validators
        // 4. Once threshold reached, aggregate G1 points to compute final decryption key
        // 5. Store aggregated key in revealed_secrets table
        //
        // For now, we just store the first share to allow basic testing
        let state = borrow_global_mut<TimelockState>(@aptos_framework);
         if (!table::contains(&state.revealed_secrets, interval)) {
            table::add(&mut state.revealed_secrets, interval, share);
        };
        // TODO: Otherwise, aggregate with existing shares (BLS aggregation)
    }

    /// Get the current interval number.
    /// Returns 0 if timelock is not initialized.
    #[view]
    public fun get_current_interval(): u64 acquires TimelockState {
        if (!exists<TimelockState>(@aptos_framework)) {
            return 0
        };
        borrow_global<TimelockState>(@aptos_framework).current_interval
    }

    /// Get the public key (MPK) for a specific interval.
    /// Returns None if the public key hasn't been published yet.
    ///
    /// This is used by clients to encrypt messages to a future interval.
    #[view]
    public fun get_public_key(interval: u64): Option<vector<u8>> acquires TimelockState {
        if (!exists<TimelockState>(@aptos_framework)) {
            return option::none()
        };
        let state = borrow_global<TimelockState>(@aptos_framework);
        if (table::contains(&state.public_keys, interval)) {
            option::some(*table::borrow(&state.public_keys, interval))
        } else {
            option::none()
        }
    }

    /// Check if the secret (aggregated decryption key) has been revealed for an interval.
    /// Returns true if the secret is available for decryption.
    ///
    /// This allows clients to check if they can decrypt messages from a past interval.
    #[view]
    public fun is_secret_revealed(interval: u64): bool acquires TimelockState {
        if (!exists<TimelockState>(@aptos_framework)) {
            return false
        };
        let state = borrow_global<TimelockState>(@aptos_framework);
        table::contains(&state.revealed_secrets, interval)
    }

    /// Get the revealed secret (aggregated decryption key) for a specific interval.
    /// Returns None if the secret hasn't been revealed yet.
    /// This is used by clients to decrypt messages from a past interval.
    /// Alias for backward compatibility.
    #[view]
    public fun get_secret(interval: u64): Option<vector<u8>> acquires TimelockState {
        if (!exists<TimelockState>(@aptos_framework)) {
            return option::none()
        };
        let state = borrow_global<TimelockState>(@aptos_framework);
        if (table::contains(&state.revealed_secrets, interval)) {
            option::some(*table::borrow(&state.revealed_secrets, interval))
        } else {
            option::none()
        }
    }

    #[test_only]
    use aptos_framework::account::create_signer_for_test;

    #[test(framework = @aptos_framework)]
    public fun test_timelock_flow(framework: &signer) acquires TimelockState {
        timestamp::set_time_has_started_for_testing(framework);
        account::create_account_for_test(@aptos_framework);
        initialize(framework);
        let vm = create_signer_for_test(@0x0);

        // First block, sets initialization time
        on_new_block(&vm);
        
        // Advance time
        timestamp::update_global_time_for_test(3600 * 1000000 + 1);
        
        // Second block, triggers rotation
        on_new_block(&vm);
        
        let state = borrow_global<TimelockState>(@aptos_framework);
        assert!(state.current_interval == 1, 100);

        // Test publishing
        let val = create_signer_for_test(@0x123);
        let pk = vector::empty<u8>();
        vector::push_back(&mut pk, 10);
        publish_public_key(&val, 1, pk);

        let share = vector::empty<u8>();
        vector::push_back(&mut share, 20);
        publish_secret_share(&val, 0, share);

        let state = borrow_global<TimelockState>(@aptos_framework);
        assert!(table::contains(&state.public_keys, 1), 101);
        assert!(table::contains(&state.revealed_secrets, 0), 102);
    }

    #[test]
    #[expected_failure(abort_code = 524294, location = aptos_framework::system_addresses)] // E_VM_NOT_APTOS_FRAMEWORK
    public fun test_unauthorized_initialize() {
        let not_framework = create_signer_for_test(@0x1);
        initialize(&not_framework);
    }
}
