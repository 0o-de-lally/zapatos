
<a id="0x1_ibe"></a>

# Module `0x1::ibe`

This module provides Identity-Based Encryption (IBE) decryption capabilities.
It uses the <code><a href="crypto_algebra.md#0x1_crypto_algebra">crypto_algebra</a></code> module for underlying algebraic structures (G1, G2, Gt).


-  [Function `decrypt`](#0x1_ibe_decrypt)
-  [Function `decrypt_internal`](#0x1_ibe_decrypt_internal)


<pre><code><b>use</b> <a href="crypto_algebra.md#0x1_crypto_algebra">0x1::crypto_algebra</a>;
</code></pre>



<a id="0x1_ibe_decrypt"></a>

## Function `decrypt`

Decrypts a message using Identity-Based Encryption (IBE) logic.
Performs Pairing(u, sig) -> Gt, Serializes Gt, Hashes (Keccak256), and XORs with ciphertext.

generic types G1, G2, Gt must match the curves used (e.g. BLS12-381).


<pre><code><b>public</b> <b>fun</b> <a href="ibe.md#0x1_ibe_decrypt">decrypt</a>&lt;G1, G2, Gt&gt;(u: &<a href="crypto_algebra.md#0x1_crypto_algebra_Element">crypto_algebra::Element</a>&lt;G1&gt;, sig: &<a href="crypto_algebra.md#0x1_crypto_algebra_Element">crypto_algebra::Element</a>&lt;G2&gt;, ciphertext: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ibe.md#0x1_ibe_decrypt">decrypt</a>&lt;G1, G2, Gt&gt;(u: &Element&lt;G1&gt;, sig: &Element&lt;G2&gt;, ciphertext: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    // Use <b>native</b> IBE decryption which is gas-optimized
    // Calls <a href="crypto_algebra.md#0x1_crypto_algebra_handle">crypto_algebra::handle</a> explicitly <b>to</b> avoid dot-call resolution issues
    <a href="ibe.md#0x1_ibe_decrypt_internal">decrypt_internal</a>&lt;G1, G2, Gt&gt;(
        <a href="crypto_algebra.md#0x1_crypto_algebra_handle">crypto_algebra::handle</a>(u),
        <a href="crypto_algebra.md#0x1_crypto_algebra_handle">crypto_algebra::handle</a>(sig),
        ciphertext
    )
}
</code></pre>



</details>

<a id="0x1_ibe_decrypt_internal"></a>

## Function `decrypt_internal`



<pre><code><b>fun</b> <a href="ibe.md#0x1_ibe_decrypt_internal">decrypt_internal</a>&lt;G1, G2, Gt&gt;(u_handle: u64, sig_handle: u64, ciphertext: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="ibe.md#0x1_ibe_decrypt_internal">decrypt_internal</a>&lt;G1, G2, Gt&gt;(u_handle: u64, sig_handle: u64, ciphertext: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;;
</code></pre>



</details>


[move-book]: https://aptos.dev/move/book/SUMMARY
