use std::{fs, io::Write, path::PathBuf};
use tokio;

const DEFINITIONS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/definitions.ts"));
const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/types.rs"));

const OUTPUT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/wasm/types.rs");

const CONFIG_BASE_URI: &str = "https://nomad-xyz.github.io/config";
const ENVS: &[&str] = &["development", "staging", "production"];
const CONFIG_BASE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/configs");

fn env_json(env: &str) -> String {
    format!("{}.json", env)
}

fn config_uri(env: &str) -> String {
    format!("{}/{}", CONFIG_BASE_URI, env_json(env))
}

fn config_path(env: &str) -> PathBuf {
    PathBuf::from(CONFIG_BASE_DIR).join(env_json(env))
}

async fn fetch_config(env: &str) -> eyre::Result<String> {
    let uri = config_uri(env);
    Ok(reqwest::get(uri).await?.text().await?)
}

fn store_config(env: &str, contents: &str) -> eyre::Result<()> {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(config_path(env))
        .unwrap();

    f.write_all(contents.as_ref())?;
    Ok(())
}

fn gen_wasm_bindgen() -> eyre::Result<()> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(OUTPUT_FILE)
        .unwrap();

    writeln!(f, "//! THIS IS AUTOGENERATED CODE, DO NOT EDIT")?;
    writeln!(
        f,
        "//! Please edit `data/definitions.ts` and `data/types.rs`"
    )?;
    writeln!(f, "use wasm_bindgen::prelude::*;")?;
    writeln!(
        f,
        r###"
#[wasm_bindgen(typescript_custom_section)]
const _: &'static str = r#""###
    )?;
    f.write_all(DEFINITIONS.as_ref())?;
    writeln!(f, r###""#;"###)?;
    writeln!(f)?;
    f.write_all(TYPEDEFS.as_ref())?;

    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    println!(
        "cargo:rerun-if-changed={}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/data/definitions.ts")
    );
    println!(
        "cargo:rerun-if-changed={}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/data/types.rs")
    );

    let (first, second, third) = tokio::join!(
        fetch_config(ENVS[0]),
        fetch_config(ENVS[1]),
        fetch_config(ENVS[2]),
    );

    if let Ok(first) = first {
        store_config(ENVS[0], &first)?
    }
    if let Ok(second) = second {
        store_config(ENVS[1], &second)?
    }
    if let Ok(third) = third {
        store_config(ENVS[2], &third)?;
    }

    for env in ENVS {
        let text = fetch_config(env).await?;
        store_config(env, &text)?;
    }

    gen_wasm_bindgen()?;

    Ok(())
}
