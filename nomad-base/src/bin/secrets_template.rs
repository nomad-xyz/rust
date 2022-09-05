use nomad_types::NameOrDomain;
use serde_json::json;
use std::{fs::OpenOptions, io::Write};

#[allow(dead_code)]
fn main() {
    let env = std::env::var("RUN_ENV").expect("missing RUN_ENV env var");
    let config = nomad_xyz_configuration::get_builtin(&env).expect("!config");

    let mut template = json!({
        "rpcs": {},
        "transactionSigners": {},
        "attestationSigner": {
            "type": "hexKey",
            "key": "",
        },
    });

    for network in config.networks.iter() {
        let networks = config.protocol();
        let rpc_style = networks
            .networks
            .get(network.as_str())
            .expect("!no domain")
            .rpc_style;
        template["rpcs"].as_object_mut().unwrap().insert(
            network.to_owned(),
            json!({
                "rpcStyle": serde_json::to_string(&rpc_style).expect("!rpcStyle"),
                "connection": {
                    "type": "http",
                    "url": ""
                  },
            }),
        );

        template["transactionSigners"]
            .as_object_mut()
            .unwrap()
            .insert(
                network.to_owned(),
                json!({
                    "type": "hexKey",
                    "key": ""
                }),
            );
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("secrets.json")
        .expect("Failed to open/create file");

    file.write_all(template.to_string().as_bytes())
        .expect("Failed to write to file");
}
