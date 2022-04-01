//! Configuration

use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::processor::ProcessorConfig;

decl_settings!(Processor, ProcessorConfig,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::NomadAgent;
    use nomad_xyz_configuration::AgentSecrets;

    #[test]
    fn it_builds_settings_from_env() {
        dotenv::from_filename("../../fixtures/env.test").unwrap();
        let run_env = dotenv::var("RUN_ENV").unwrap();
        let agent_home = dotenv::var("AGENT_HOME").unwrap();

        let settings = ProcessorSettings::new().unwrap();

        let config = nomad_xyz_configuration::get_builtin(&run_env).unwrap();
        let secrets = AgentSecrets::from_env(&config.networks).unwrap();

        settings
            .base
            .validate_against_config_and_secrets(
                crate::Processor::AGENT_NAME,
                &agent_home,
                config,
                &secrets,
            )
            .unwrap();

        let agent_config = &config.agent().get("ethereum").unwrap().processor;
        assert_eq!(settings.agent.interval, agent_config.interval);
        assert_eq!(settings.agent.allowed, agent_config.allowed);
        assert_eq!(settings.agent.denied, agent_config.denied);
        assert_eq!(
            settings.agent.subsidized_remotes,
            agent_config.subsidized_remotes
        );
        assert_eq!(settings.agent.s3, agent_config.s3);
    }
}
