use nomad_core::SignedUpdate;
use subxt::ext::scale_value::{Primitive, Value};

/// Format signed update into scale value format
pub fn format_signed_update_value(signed_update: &SignedUpdate) -> Value {
    let SignedUpdate { update, signature } = signed_update;

    Value::named_composite([
        (
            "update",
            Value::named_composite([
                ("home_domain", Value::u128(update.home_domain as u128)),
                ("previous_root", Value::from_bytes(&update.previous_root)),
                ("new_root", Value::from_bytes(&update.new_root)),
            ]),
        ),
        (
            "signature",
            Value::named_composite([
                ("r", Value::primitive(Primitive::U256(signature.r.into()))),
                ("s", Value::primitive(Primitive::U256(signature.s.into()))),
                ("v", Value::u128(signature.v as u128)),
            ]),
        ),
    ])
}
