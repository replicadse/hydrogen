use std::{
    error::Error,
    result::Result,
};

use crate::error::UnknownCommandError;

#[derive(Debug)]
pub struct CallArgs {
    pub command: Command,
}

impl CallArgs {
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[derive(Debug)]
/// The (sub-)command representation for the call args.
pub enum Command {
    Work { config: crate::config::Config, publish: bool },
}

/// The type that parses the arguments to the program.
pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    /// Parsing the program arguments with the `clap` trait.
    pub fn load() -> Result<CallArgs, Box<dyn Error>> {
        let command = clap::App::new("spoderman")
            .version(env!("CARGO_PKG_VERSION"))
            .about("spoderman")
            .author("replicadse <aw@voidpointergroup.com>")
            .arg(
                clap::Arg::new("experimental")
                    .short('e')
                    .long("experimental")
                    .value_name("EXPERIMENTAL")
                    .help("Enables experimental features that do not count as stable.")
                    .required(false)
                    .takes_value(false),
            )
            .subcommand(
                clap::App::new("work").about("").arg(
                    clap::Arg::new("config")
                        .short('c')
                        .long("config")
                        .value_name("CONFIG")
                        .help("The configuration file to use.")
                        .default_value("./config.yaml")
                        .multiple_occurrences(false)
                        .multiple_values(false)
                        .required(false)
                        .takes_value(true),
                ).arg(
                    clap::Arg::new("publish")
                        .short('p')
                        .long("publish")
                        .value_name("PUBLISH")
                        .required(false)
                        .takes_value(false),
                ),
            )
            .get_matches();

        let cmd = if let Some(x) = command.subcommand_matches("work") {
            let config_content = if x.is_present("config") {
                let config_param = x.value_of("config").unwrap();
                std::fs::read_to_string(config_param)?
            } else {
                return Err(Box::new(crate::error::MissingArgumentError::new(
                    "configuration unspecified",
                )));
            };
            Command::Work {
                config: serde_yaml::from_str(&config_content)?,
                publish: x.is_present("publish")
            }
        } else {
            return Err(Box::new(UnknownCommandError::new("unknown command")));
        };

        let callargs = CallArgs { command: cmd };

        callargs.validate()?;
        Ok(callargs)
    }
}
