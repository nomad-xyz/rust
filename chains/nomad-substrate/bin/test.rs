use nomad_substrate::AvailConfig;
use subxt::ext::scale_value::Value;
use subxt::OnlineClient;

#[tokio::main]
async fn main() {
    let api = OnlineClient::<AvailConfig>::new().await.unwrap();

    let signer = subxt::tx::PairSigner::new(sp_keyring::AccountKeyring::Alice.pair());
    let dest = sp_keyring::AccountKeyring::Bob.to_account_id();
    let tx = subxt::dynamic::tx(
        "Balances",
        "transfer",
        vec![
            // A value representing a MultiAddress<AccountId32, _>. We want the "Id" variant, and that
            // will ultimately contain the bytes for our destination address (there is a new type wrapping
            // the address, but our encoding will happily ignore such things and do it's best to line up what
            // we provide with what it needs).
            Value::unnamed_variant("Id", [Value::from_bytes(&dest)]),
            // A value representing the amount we'd like to transfer.
            Value::u128(1u128),
        ],
    );

    api.tx()
        .sign_and_submit_default(&tx, &signer)
        .await
        .unwrap();
}
