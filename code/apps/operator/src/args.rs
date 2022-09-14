use std::result::Result;

#[derive(Debug)]
pub struct CallArgs {
    pub command: Command,
}

impl CallArgs {
    pub fn validate(&self) -> Result<(), crate::error::WKError> {
        Ok(())
    }
}

#[derive(Debug)]
/// The (sub-)command representation for the call args.
pub enum Command {
    Exec { config: crate::config::Config },
}

/// The type that parses the arguments to the program.
pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    /// Parsing the program arguments with the `clap` trait.
    pub fn load() -> Result<CallArgs, crate::error::WKError> {
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
                clap::App::new("exec").about("").arg(
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

        let cmd = if let Some(x) = command.subcommand_matches("exec") {
            let config_content = if x.is_present("config") {
                let config_param = x.value_of("config").unwrap();
                std::fs::read_to_string(config_param).or_else(|_| Err(crate::error::WKError::Unknown))?
            } else {
                return Err(crate::error::WKError::MissingArgument(
                    "configuration unspecified".to_owned(),
                ));
            };
            Command::Exec {
                config: serde_yaml::from_str(&config_content).or_else(|_| Err(crate::error::WKError::Unknown))?,
            }
        } else {
            return Err(crate::error::WKError::UnknownCommand("unknown command".to_owned()));
        };

        let callargs = CallArgs { command: cmd };

        callargs.validate()?;
        Ok(callargs)
    }
}
