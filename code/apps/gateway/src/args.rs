use std::{
    error::Error,
    result::Result,
};

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
    Serve { config: crate::config::Config },
}

/// The type that parses the arguments to the program.
pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    /// Parsing the program arguments with the `clap` trait.
    pub fn load() -> Result<CallArgs, Box<dyn Error>> {
        let command = clap::App::new("hydrogen")
            .version(env!("CARGO_PKG_VERSION"))
            .about("hydrogen")
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
                clap::App::new("serve").about("").arg(
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
                ),
            )
            .get_matches();

        let cmd = if let Some(x) = command.subcommand_matches("serve") {
            let config_content = if x.is_present("config") {
                let config_param = x.value_of("config").unwrap();
                std::fs::read_to_string(config_param)?
            } else {
                return Err(Box::new(crate::error::MissingArgumentError::new(
                    "configuration unspecified",
                )));
            };
            Command::Serve {
                config: serde_yaml::from_str(&config_content)?,
            }
        } else {
            return Err(Box::new(crate::error::UnknownCommandError::new("unknown command")));
        };

        let callargs = CallArgs { command: cmd };

        callargs.validate()?;
        Ok(callargs)
    }
}
