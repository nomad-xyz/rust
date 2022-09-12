use crate::SubstrateError;
use ethers_core::types::H256;
use nomad_core::SignedUpdate;
use subxt::{
    client::OnlineClientT,
    ext::scale_value::{Primitive, Value},
    tx::TxEvents,
    tx::TxInBlock,
    Config,
};

/// Try to convert `TxInBlock` to `TxEvents`, which can only happen if tx
/// in block succeeds. Attempt to catch module errors as determinstic reverts.
pub async fn try_tx_in_block_to_successful_tx_events<T, C>(
    tx_in_block: TxInBlock<T, C>,
) -> Result<TxEvents<T>, SubstrateError>
where
    T: Config,
    C: OnlineClientT<T>,
    <T as Config>::Hash: Into<H256>,
{
    // Try to detect reverting txs that were submitted to chain
    tx_in_block.wait_for_success().await.map_err(|err| {
        if let subxt::Error::Runtime(subxt::error::DispatchError::Module(_)) = err {
            return SubstrateError::TxNotExecuted(tx_in_block.extrinsic_hash().into());
        }

        SubstrateError::ProviderError(err)
    })
}

/// Format signed update into scale value format
pub fn format_signed_update_value(signed_update: &SignedUpdate) -> Value {
    let SignedUpdate { update, signature } = signed_update;

    let r_bytes = signature.r.0;
    let s_bytes = signature.s.0;

    Value::named_composite([
        (
            "update",
            Value::named_composite([
                ("home_domain", Value::u128(update.home_domain as u128)),
                (
                    "previous_root",
                    Value::primitive(Primitive::U256(update.previous_root.into())),
                ),
                (
                    "new_root",
                    Value::primitive(Primitive::U256(update.new_root.into())),
                ),
            ]),
        ),
        (
            "signature",
            Value::named_composite([
                (
                    "r",
                    Value::unnamed_composite([
                        Value::u128(r_bytes[0] as u128),
                        Value::u128(r_bytes[1] as u128),
                        Value::u128(r_bytes[2] as u128),
                        Value::u128(r_bytes[3] as u128),
                    ]),
                ),
                (
                    "s",
                    Value::unnamed_composite([
                        Value::u128(s_bytes[0] as u128),
                        Value::u128(s_bytes[1] as u128),
                        Value::u128(s_bytes[2] as u128),
                        Value::u128(s_bytes[3] as u128),
                    ]),
                ),
                ("v", Value::u128(signature.v as u128)),
            ]),
        ),
    ])
}
