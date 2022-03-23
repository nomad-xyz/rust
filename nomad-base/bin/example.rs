use color_eyre::Result;

// use nomad_base::{decl_settings, NomadAgent};

// /// Example agent
// pub struct Example;

// // Example settings block containing addition agent-specific fields
// #[derive(Debug, Clone, serde::Deserialize)]
// pub struct ExampleConfig {
//     interval: u64,
//     enabled: bool,
// }

// // Generate ExampleSettings which includes base and agent-specific settings
// decl_settings!(Example, ExampleConfig,);

// /// An example main function for any agent that implemented Default
// async fn _example_main<NA>(settings: ExampleSettings) -> Result<()>
// where
//     NA: NomadAgent<Settings = ExampleSettings> + Sized + 'static,
// {
//     // Instantiate an agent
//     let oa = NA::from_settings(settings).await?;
//     oa.start_tracing(oa.metrics().span_duration())?;

//     // Use the agent to run a number of replicas
//     oa.run_all().await?
// }

// /// Read settings from the config file and set up reporting and logging based
// /// on the settings
// #[allow(dead_code)]
// fn setup() -> Result<ExampleSettings> {
//     color_eyre::install()?;

//     let settings = ExampleSettings::new()?;

//     Ok(settings)
// }

// #[allow(dead_code)]
fn main() -> Result<()> {
    //     // let _settings = setup()?;
    //     // tokio::runtime::Builder::new_current_thread()
    //     //     .enable_all()
    //     //     .build()
    //     //     .unwrap()
    //     //     .block_on(_example_main(settings))?;

    Ok(())
}
