
<a id="0x1_repro_deserialize"></a>

# Module `0x1::repro_deserialize`



-  [Resource `Noop`](#0x1_repro_deserialize_Noop)
-  [Function `should_not_abort`](#0x1_repro_deserialize_should_not_abort)
-  [Function `maybe_aborts`](#0x1_repro_deserialize_maybe_aborts)
-  [Function `should_init_struct`](#0x1_repro_deserialize_should_init_struct)


<pre><code><b>use</b> <a href="aptos_account.md#0x1_aptos_account">0x1::aptos_account</a>;
<b>use</b> <a href="../../aptos-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
</code></pre>



<a id="0x1_repro_deserialize_Noop"></a>

## Resource `Noop`



<pre><code><b>struct</b> <a href="repro_debugger_deserialize.md#0x1_repro_deserialize_Noop">Noop</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_repro_deserialize_should_not_abort"></a>

## Function `should_not_abort`



<pre><code><b>public</b> <b>fun</b> <a href="repro_debugger_deserialize.md#0x1_repro_deserialize_should_not_abort">should_not_abort</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="repro_debugger_deserialize.md#0x1_repro_deserialize_should_not_abort">should_not_abort</a>() {
  <b>let</b> a = <b>exists</b>&lt;<a href="repro_debugger_deserialize.md#0x1_repro_deserialize_Noop">Noop</a>&gt;(@0xabc);
  print(&a);
}
</code></pre>



</details>

<a id="0x1_repro_deserialize_maybe_aborts"></a>

## Function `maybe_aborts`



<pre><code><b>public</b> entry <b>fun</b> <a href="repro_debugger_deserialize.md#0x1_repro_deserialize_maybe_aborts">maybe_aborts</a>(addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="repro_debugger_deserialize.md#0x1_repro_deserialize_maybe_aborts">maybe_aborts</a>(addr: <b>address</b>) {
  <b>let</b> a = <b>exists</b>&lt;<a href="repro_debugger_deserialize.md#0x1_repro_deserialize_Noop">Noop</a>&gt;(addr);
  print(&a);
}
</code></pre>



</details>

<a id="0x1_repro_deserialize_should_init_struct"></a>

## Function `should_init_struct`



<pre><code><b>public</b> entry <b>fun</b> <a href="repro_debugger_deserialize.md#0x1_repro_deserialize_should_init_struct">should_init_struct</a>(sig: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="repro_debugger_deserialize.md#0x1_repro_deserialize_should_init_struct">should_init_struct</a>(sig: &<a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
  <b>let</b> addr = <a href="../../aptos-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(sig);
  <a href="aptos_account.md#0x1_aptos_account_create_account">aptos_account::create_account</a>(addr);
  <b>if</b> (!<b>exists</b>&lt;<a href="repro_debugger_deserialize.md#0x1_repro_deserialize_Noop">Noop</a>&gt;(addr)) {
    print(&addr);
    <b>move_to</b>&lt;<a href="repro_debugger_deserialize.md#0x1_repro_deserialize_Noop">Noop</a>&gt;(sig, <a href="repro_debugger_deserialize.md#0x1_repro_deserialize_Noop">Noop</a> {});
  }
}
</code></pre>



</details>


[move-book]: https://aptos.dev/move/book/SUMMARY
