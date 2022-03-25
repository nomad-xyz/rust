use serde_json::json;
use std::{fs::OpenOptions, io::Write};

fn main() {
    let env = std::env::var("RUN_ENV").expect("missing RUN_ENV env var");
    output_overridable_config(&env);
    generate_secrets_template(&env);
}

fn output_overridable_config(env: &str) {
    let json = match env {
        "test" => nomad_xyz_configuration::builtin::TEST_JSON,
        "development" => nomad_xyz_configuration::builtin::DEVELOPMENT_JSON,
        "staging" => nomad_xyz_configuration::builtin::STAGING_JSON,
        "production" => nomad_xyz_configuration::builtin::PRODUCTION_JSON,
        _ => panic!("unknown environment {}", env),
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("config.json")
        .expect("Failed to open/create config.json");

    file.write_all(json.as_bytes())
        .expect("Failed to write to config.json");
}

fn generate_secrets_template(env: &str) {
    let config = nomad_xyz_configuration::get_builtin(env).expect("!config");

    let mut template = json!({
        "rpcs": {},
        "transactionSigners": {},
        "attestationSigner": {
            "type": "",
            "key": "",
        },
    });

    for network in config.networks.iter() {
        let rpc_style = config.agent().get(network).expect("!agent").rpc_style;
        template["rpcs"].as_object_mut().unwrap().insert(
            network.to_owned(),
            json!({
                "rpcStyle": serde_json::to_string(&rpc_style).expect("!rpcStyle"),
                "connection": {
                    "type": "",
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
                    "type": "",
                    "key": ""
                }),
            );
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("secrets.json")
        .expect("Failed to open/create secrets.json");

    file.write_all(template.to_string().as_bytes())
        .expect("Failed to write to secrets.json");
}
