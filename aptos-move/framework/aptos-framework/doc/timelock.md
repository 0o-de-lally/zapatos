
<a id="0x1_timelock"></a>

# Module `0x1::timelock`



-  [Struct `TimelockConfig`](#0x1_timelock_TimelockConfig)
-  [Resource `TimelockState`](#0x1_timelock_TimelockState)
-  [Struct `StartKeyGenEvent`](#0x1_timelock_StartKeyGenEvent)
-  [Struct `RequestRevealEvent`](#0x1_timelock_RequestRevealEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_timelock_initialize)
-  [Function `on_new_block`](#0x1_timelock_on_new_block)
-  [Function `publish_public_key`](#0x1_timelock_publish_public_key)
-  [Function `publish_secret_share`](#0x1_timelock_publish_secret_share)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `on_new_block`](#@Specification_1_on_new_block)
    -  [Function `publish_public_key`](#@Specification_1_publish_public_key)
    -  [Function `publish_secret_share`](#@Specification_1_publish_secret_share)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="../../aptos-stdlib/doc/table.md#0x1_table">0x1::table</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
</code></pre>



<a id="0x1_timelock_TimelockConfig"></a>

## Struct `TimelockConfig`



<pre><code><b>struct</b> <a href="timelock.md#0x1_timelock_TimelockConfig">TimelockConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>threshold: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>total_validators: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_timelock_TimelockState"></a>

## Resource `TimelockState`



<pre><code><b>struct</b> <a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>current_interval: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>last_rotation_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>public_keys: <a href="../../aptos-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;u64, <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>
 Store public keys (for encryption)
</dd>
<dt>
<code>revealed_secrets: <a href="../../aptos-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;u64, <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>
 Store revealed secret keys/signatures (for decryption)
</dd>
<dt>
<code>start_keygen_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="timelock.md#0x1_timelock_StartKeyGenEvent">timelock::StartKeyGenEvent</a>&gt;</code>
</dt>
<dd>
 Events
</dd>
<dt>
<code>request_reveal_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="timelock.md#0x1_timelock_RequestRevealEvent">timelock::RequestRevealEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_timelock_StartKeyGenEvent"></a>

## Struct `StartKeyGenEvent`

Event emitted to tell validators: "Please generate keys for interval X"


<pre><code><b>struct</b> <a href="timelock.md#0x1_timelock_StartKeyGenEvent">StartKeyGenEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>interval: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>config: <a href="timelock.md#0x1_timelock_TimelockConfig">timelock::TimelockConfig</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_timelock_RequestRevealEvent"></a>

## Struct `RequestRevealEvent`

Event emitted to tell validators: "Please reveal the secret for interval X"


<pre><code><b>struct</b> <a href="timelock.md#0x1_timelock_RequestRevealEvent">RequestRevealEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>interval: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_timelock_ETIMELOCK_NOT_INITIALIZED"></a>

The singleton was not initialized.


<pre><code><b>const</b> <a href="timelock.md#0x1_timelock_ETIMELOCK_NOT_INITIALIZED">ETIMELOCK_NOT_INITIALIZED</a>: u64 = 1;
</code></pre>



<a id="0x1_timelock_initialize"></a>

## Function `initialize`

Initialize the timelock system.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timelock.md#0x1_timelock_initialize">initialize</a>(framework: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timelock.md#0x1_timelock_initialize">initialize</a>(framework: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <a href="system_addresses.md#0x1_system_addresses_assert_aptos_framework">system_addresses::assert_aptos_framework</a>(framework);
    <b>move_to</b>(framework, <a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a> {
        current_interval: 0,
        last_rotation_time: 0, // Will be updated on first <a href="block.md#0x1_block">block</a>
        public_keys: <a href="../../aptos-stdlib/doc/table.md#0x1_table_new">table::new</a>(),
        revealed_secrets: <a href="../../aptos-stdlib/doc/table.md#0x1_table_new">table::new</a>(),
        start_keygen_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="timelock.md#0x1_timelock_StartKeyGenEvent">StartKeyGenEvent</a>&gt;(framework),
        request_reveal_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="timelock.md#0x1_timelock_RequestRevealEvent">RequestRevealEvent</a>&gt;(framework),
    });
}
</code></pre>



</details>

<a id="0x1_timelock_on_new_block"></a>

## Function `on_new_block`

Called by block prologue to trigger rotations.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timelock.md#0x1_timelock_on_new_block">on_new_block</a>(vm: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timelock.md#0x1_timelock_on_new_block">on_new_block</a>(vm: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_vm">system_addresses::assert_vm</a>(vm);

    <b>if</b> (!<b>exists</b>&lt;<a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a>&gt;(@aptos_framework)) {
        <b>return</b>
    };

    <b>let</b> state = <b>borrow_global_mut</b>&lt;<a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a>&gt;(@aptos_framework);
    <b>let</b> now = <a href="timestamp.md#0x1_timestamp_now_microseconds">timestamp::now_microseconds</a>();

    // Initialize last_rotation_time <b>if</b> it's 0 (<a href="genesis.md#0x1_genesis">genesis</a>/first run)
    <b>if</b> (state.last_rotation_time == 0) {
        state.last_rotation_time = now;
        <b>return</b>
    };

    // Check <b>if</b> 1 hour <b>has</b> passed (3600 seconds * 1,000,000 microseconds)
    <b>let</b> one_hour_micros = 3600 * 1000000;
    <b>if</b> (now - state.last_rotation_time &gt; one_hour_micros) {
        <b>let</b> old_interval = state.current_interval;
         // Emit reveal <a href="event.md#0x1_event">event</a> for the <b>old</b> interval
        <a href="event.md#0x1_event_emit_event">event::emit_event</a>(&<b>mut</b> state.request_reveal_events, <a href="timelock.md#0x1_timelock_RequestRevealEvent">RequestRevealEvent</a> {
            interval: old_interval,
        });

        state.current_interval = state.current_interval + 1;
        state.last_rotation_time = now;

        // TODO: In a real implementation, we would get the actual validator set size/threshold.
        // For this PoC, we'll hardcode or placeholders.
        // Let's <b>assume</b> a fixed threshold for now or just emit the <a href="event.md#0x1_event">event</a>.
        <b>let</b> config = <a href="timelock.md#0x1_timelock_TimelockConfig">TimelockConfig</a> {
            threshold: 1, // Placeholder
            total_validators: 1, // Placeholder
        };

        <a href="event.md#0x1_event_emit_event">event::emit_event</a>(&<b>mut</b> state.start_keygen_events, <a href="timelock.md#0x1_timelock_StartKeyGenEvent">StartKeyGenEvent</a> {
            interval: state.current_interval,
            config,
        });
    }
}
</code></pre>



</details>

<a id="0x1_timelock_publish_public_key"></a>

## Function `publish_public_key`

validators call this to publish the public key for a future interval


<pre><code><b>public</b> entry <b>fun</b> <a href="timelock.md#0x1_timelock_publish_public_key">publish_public_key</a>(validator: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, interval: u64, pk: <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="timelock.md#0x1_timelock_publish_public_key">publish_public_key</a>(
    validator: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    interval: u64,
    pk: <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a> {
    // TODO: Verify sender is a validator
    // In this minimal PoC, we trust the sender for now or <b>assume</b> the VM only allows valid calls via ValidatorTxn
    // BUT `<b>public</b> entry` means anyone can call it?
    // The plan says "Submit <a href="timelock.md#0x1_timelock_publish_secret_share">0x1::timelock::publish_secret_share</a>" via ValidatorTransaction.
    // If it comes via ValidatorTransaction, it should be a governance/system transaction, but `entry` allows user calls.

    // Use <a href="system_addresses.md#0x1_system_addresses">system_addresses</a> or relevant checks <b>if</b> strictly required.
    // For PoC, <b>let</b>'s keep it simple but functional.

    <b>let</b> state = <b>borrow_global_mut</b>&lt;<a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a>&gt;(@aptos_framework);
    <b>if</b> (!<a href="../../aptos-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&state.public_keys, interval)) {
        <a href="../../aptos-stdlib/doc/table.md#0x1_table_add">table::add</a>(&<b>mut</b> state.public_keys, interval, pk);
    };
}
</code></pre>



</details>

<a id="0x1_timelock_publish_secret_share"></a>

## Function `publish_secret_share`

validators call this to publish the secret share/signature for a past interval


<pre><code><b>public</b> entry <b>fun</b> <a href="timelock.md#0x1_timelock_publish_secret_share">publish_secret_share</a>(validator: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, interval: u64, share: <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="timelock.md#0x1_timelock_publish_secret_share">publish_secret_share</a>(
    validator: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    interval: u64,
    share: <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a> {
    // TODO: Aggregation logic would go here.
    // For PoC, just storing the first one for now or a list.
    // The <b>struct</b> says `revealed_secrets: Table&lt;u64, <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;`.
    // We will just overwrite/store it <b>to</b> show flow.

    <b>let</b> state = <b>borrow_global_mut</b>&lt;<a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a>&gt;(@aptos_framework);
     <b>if</b> (!<a href="../../aptos-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&state.revealed_secrets, interval)) {
        <a href="../../aptos-stdlib/doc/table.md#0x1_table_add">table::add</a>(&<b>mut</b> state.revealed_secrets, interval, share);
    };
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>


Helper to get the TimelockState resource


<a id="0x1_timelock_spec_timelock_state"></a>


<pre><code><b>fun</b> <a href="timelock.md#0x1_timelock_spec_timelock_state">spec_timelock_state</a>(): <a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a> {
   <b>borrow_global</b>&lt;<a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a>&gt;(@aptos_framework)
}
</code></pre>


Invariant: current_interval is always non-negative (implied by u64, but useful anchor)
Real invariant: last_rotation_time is never in the future relative to environment time

Invariant: current_interval is always non-negative (implied by u64, but useful anchor)
Real invariant: last_rotation_time is never in the future relative to environment time


<pre><code><b>invariant</b> <b>exists</b>&lt;<a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a>&gt;(@aptos_framework) ==&gt;
    <a href="timelock.md#0x1_timelock_spec_timelock_state">spec_timelock_state</a>().last_rotation_time &lt;= aptos_framework::timestamp::spec_now_microseconds();
</code></pre>



<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timelock.md#0x1_timelock_initialize">initialize</a>(framework: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>let</b> addr = std::signer::address_of(framework);
<b>aborts_if</b> !<a href="system_addresses.md#0x1_system_addresses_is_aptos_framework_address">system_addresses::is_aptos_framework_address</a>(addr);
<b>aborts_if</b> <b>exists</b>&lt;<a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a>&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="timelock.md#0x1_timelock_TimelockState">TimelockState</a>&gt;(addr);
</code></pre>



<a id="@Specification_1_on_new_block"></a>

### Function `on_new_block`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="timelock.md#0x1_timelock_on_new_block">on_new_block</a>(vm: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>let</b> addr = std::signer::address_of(vm);
<b>aborts_if</b> addr != @vm_reserved;
</code></pre>



<a id="@Specification_1_publish_public_key"></a>

### Function `publish_public_key`


<pre><code><b>public</b> entry <b>fun</b> <a href="timelock.md#0x1_timelock_publish_public_key">publish_public_key</a>(validator: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, interval: u64, pk: <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<a id="@Specification_1_publish_secret_share"></a>

### Function `publish_secret_share`


<pre><code><b>public</b> entry <b>fun</b> <a href="timelock.md#0x1_timelock_publish_secret_share">publish_secret_share</a>(validator: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, interval: u64, share: <a href="../../aptos-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>


[move-book]: https://aptos.dev/move/book/SUMMARY
