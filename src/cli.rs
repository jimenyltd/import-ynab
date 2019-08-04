use crate::prelude::*;

pub mod revolut;
pub mod truelayer;

pub fn run() -> Result<(), Error> {
    let args: SyncYnab = SyncYnab::from_args();

    match args.command {
        SyncYnabCommands::Revolut(n) => revolut::handle(n),
        SyncYnabCommands::Truelayer(n) => truelayer::handle(n),
        SyncYnabCommands::Config(n) => config::handle(args.args, n),
        SyncYnabCommands::Sync(_n) => {
            crate::ynab::sync(&mut crate::config::load_config(args.args.config_directory)?)
        }
    }
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SyncYnab {
    #[structopt(flatten)]
    pub args: SyncYnabArgs,
    #[structopt(subcommand)]
    pub command: SyncYnabCommands,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SyncYnabArgs {
    #[structopt(long, default_value = "secrets/")]
    pub config_directory: String,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum SyncYnabCommands {
    Revolut(revolut::RevolutClient),
    Truelayer(truelayer::TruelayerArgs),
    Config(config::ConfigCommands),
    Sync(sync::SyncArgs),
}

pub mod config {
    use crate::cli::SyncYnabArgs;
    use crate::config::*;
    use crate::prelude::*;
    use std::io::{stdin, BufRead};

    #[derive(StructOpt)]
    #[structopt(rename_all = "kebab-case")]
    pub enum ConfigCommands {
        TestProviders,
        TestYnab,
        AddTruelayer,
    }

    pub fn handle(args: SyncYnabArgs, command: ConfigCommands) -> Result<(), Error> {
        let mut config = crate::config::load_config(&args.config_directory)?;

        use oauth2::TokenResponse;

        match command {
            ConfigCommands::TestProviders => {
                let mut result = crate::load_connections(&mut config)?;
                for provider in &mut result {
                    let accounts = provider.as_mut().get_accounts()?;
                    println!("{:#?}", accounts);
                    for acc in accounts {
                        let trans = provider.as_mut().get_transactions(&acc)?;
                        println!("{} has {} transactions", acc.display_name, trans.len(),);
                    }
                }
            }
            ConfigCommands::TestYnab => {
                let mut rc = crate::ynab::new_rest_client(&config.ynab_config.access_token);
                println!(
                    "{:#?}",
                    crate::ynab::get_accounts(&mut rc, &config.ynab_config.budget_id)
                );
            }
            ConfigCommands::AddTruelayer => {
                println!(
                    "Please authenticate at:\n{}",
                    crate::truelayer::get_auth_url()?.to_string()
                );

                println!("Enter code:\n");
                for line in stdin().lock().lines() {
                    let line = line?;

                    if !line.is_empty() {
                        let token = crate::truelayer::authorize(line)?;

                        let mut token = crate::truelayer::Token {
                            display_name: "unknown".to_string(),
                            access_token: token.access_token().clone(),
                            access_token_expiry: crate::truelayer::calculate_expiry_time(
                                token.expires_in().unwrap(),
                            ),
                            refresh_token: token.refresh_token().unwrap().clone(),
                        };

                        let (_refresh, result) = crate::truelayer::initialize(&mut token);
                        result?;

                        config.providers.push(Provider::Truelayer(token));

                        break;
                    }
                }

                crate::config::save_config(&args.config_directory, &config)?;
            }
        }
        Ok(())
    }
}

pub mod sync {
    use crate::prelude::*;

    #[derive(StructOpt)]
    #[structopt(rename_all = "kebab-case")]
    pub struct SyncArgs {
        //        #[structopt(subcommand)]
    //        command: SyncCommands
    }

    //    #[derive(StructOpt)]
    //    #[structopt(rename_all = "kebab-case")]
    //    pub enum SyncCommands {
    //
    //    }
}