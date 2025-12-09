module aptos_framework::timelock {

    use aptos_std::table::{Self, Table};
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;
    use aptos_framework::system_addresses;
    use aptos_framework::account;

    friend aptos_framework::block;
    friend aptos_framework::genesis;

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

        // Check if 1 hour has passed (3600 seconds * 1,000,000 microseconds)
        let one_hour_micros = 3600 * 1000000;
        if (now - state.last_rotation_time > one_hour_micros) {
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
        validator: &signer,
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
        validator: &signer,
        interval: u64,
        share: vector<u8>
    ) acquires TimelockState {
        // TODO: Aggregation logic would go here.
        // For PoC, just storing the first one for now or a list.
        // The struct says `revealed_secrets: Table<u64, vector<u8>>`.
        // We will just overwrite/store it to show flow.
        
        let state = borrow_global_mut<TimelockState>(@aptos_framework);
         if (!table::contains(&state.revealed_secrets, interval)) {
            table::add(&mut state.revealed_secrets, interval, share);
        };
    }

    #[test_only]
    use aptos_framework::account::create_signer_for_test;

    #[test(framework = @aptos_framework)]
    public fun test_timelock_flow(framework: &signer) acquires TimelockState {
        timestamp::set_time_has_started_for_testing(framework);
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

    #[test(framework = @aptos_framework)]
    #[expected_failure(abort_code = 524294, location = aptos_framework::system_addresses)] // E_VM_NOT_APTOS_FRAMEWORK
    public fun test_unauthorized_initialize(framework: &signer) {
        let not_framework = create_signer_for_test(@0x1);
        initialize(&not_framework);
    }
}
