
use aptos_gas_schedule::gas_params::natives::aptos_framework::ACCOUNT_CREATE_SIGNER_BASE;
use aptos_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
};
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::{smallvec, SmallVec};
use std::collections::VecDeque;

use aptos_types::vm_status::StatusCode;
use aptos_native_interface::SafeNativeError;
use vdf::{VDFParams, VDF};
/***************************************************************************************************
 * native fun vdf verify
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/

 /// verify the VDF proof
pub(crate) fn native_verify(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {

    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 5);

    context.charge(ACCOUNT_CREATE_SIGNER_BASE)?; // TODO: pick a cost

    let wesolowski = safely_pop_arg!(arguments, bool); // will do pietrezak if `false`.
    let security = safely_pop_arg!(arguments, u64);
    let difficulty = safely_pop_arg!(arguments, u64);
    let solution = safely_pop_arg!(arguments, Vec<u8>);
    let challenge = safely_pop_arg!(arguments, Vec<u8>);

    // refuse to try anything with a security parameter above 2048 or a difficulty above 3_000_000_001 (which is the target on Wesolowski)
    if (security > 2048) || (difficulty > 3_000_000_001) {
      return Err(SafeNativeError::Abort {
            abort_code: StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE.into(),
        })
    }

    let result = if wesolowski {
      let v = vdf::WesolowskiVDFParams(security as u16).new();
      v.verify(&challenge, difficulty, &solution)
    } else {

      if difficulty > 900_000_000 {
        return Err(SafeNativeError::Abort {
            abort_code: StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE.into(),
        })
      }

      let v = vdf::PietrzakVDFParams(security as u16).new();
      v.verify(&challenge, difficulty, &solution)
    };

    Ok(smallvec![Value::bool(result.is_ok())])
}


// TODO: This no longer needs to be a native, since we have complete vector operations in stdlib
/// get the address contained in the first VDF challenge.
pub(crate) fn native_extract_address_from_challenge(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.is_empty());
    let challenge_vec = safely_pop_arg!(arguments, Vec<u8>);

    // We want to use Diem AuthenticationKey::derived_address() here but this creates
    // libra (and as a result cyclic) dependency which we definitely do not want
    // const AUTHENTICATION_KEY_LENGTH: usize = 64;
    // let auth_key_vec = &challenge_vec[..AUTHENTICATION_KEY_LENGTH];
    // Address derived from the last `AccountAddress::LENGTH` bytes of authentication key
    let mut array = [0u8; 32];
    array.copy_from_slice(
        &challenge_vec[..32]
    );
    let address = AccountAddress::new(array);

    context.charge(ACCOUNT_CREATE_SIGNER_BASE)?; // TODO: pick a cost

    Ok(smallvec![
      Value::signer(address),
      Value::vector_u8(array.to_vec())
    ])
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [
      ("verify", native_verify as RawSafeNative),
      ("extract_address_from_challenge", native_extract_address_from_challenge as RawSafeNative)
    ];

    builder.make_named_natives(natives)
}


#[test]
fn sanity_test_vdf() {
  let security = 512u16;
  let difficulty = 100;
  let challenge = hex::decode("aa").unwrap();
  let solution = hex::decode("0051dfa4c3341c18197b72f5e5eecc693eb56d408206c206d90f5ec7a75f833b2affb0ea7280d4513ab8351f39362d362203ff3e41882309e7900f470f0a27eeeb7b").unwrap();

  let v = vdf::PietrzakVDFParams(security).new();
  v.verify(&challenge, difficulty, &solution).unwrap();
}

#[test]
fn round_trip() {
    let pietrzak_vdf = vdf::PietrzakVDFParams(512).new();
    let solution = pietrzak_vdf.solve(b"\xaa", 100).unwrap();
    dbg!(&hex::encode(&solution));
    assert!(pietrzak_vdf.verify(b"\xaa", 100, &solution).is_ok());
}
