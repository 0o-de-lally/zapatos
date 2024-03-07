
// the Move VM when running a session aborts on evalutaing exists()==false
//  when it should only assign to false.
module aptos_framework::repro_deserialize {

use std::signer;
use aptos_std::debug::print;
use aptos_framework::aptos_account;

struct Noop has key {}

// both functions below cause a FAILED_TO_DESERIALIZE_RESOURCE since 0xabc does
// not exist yet.
// sometimes if we are generating a writeset offline (e.g. for creating new
// accounts), we won't be able to initialize any structs

public fun should_not_abort() {
  let a = exists<Noop>(@0xabc);
  print(&a);
}

// same if the arg is in the transaction
public entry fun maybe_aborts(addr: address) {
  let a = exists<Noop>(addr);
  print(&a);
}

// same if the arg is in the transaction
public entry fun should_init_struct(sig: &signer) {
  let addr = signer::address_of(sig);
  aptos_account::create_account(addr);
  if (!exists<Noop>(addr)) {
    print(&addr);
    move_to<Noop>(sig, Noop {});
  }
}

}
