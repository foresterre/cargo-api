use cargo_api::api::crates::Crate;
use cargo_api::api::{Json, Query};
use cargo_api::client::ReqwestClient;
use clap::{Args, Parser};
use std::borrow::Cow;

fn main() -> anyhow::Result<()> {
    let CargoCli::Api(args) = CargoCli::parse();

    let client = ReqwestClient::new(args.user_agent.as_str());

    match args.subcommand {
        Subcommand::Crate(opts) => {
            let endpoint = Json::new(Crate::new(Cow::Borrowed(opts.name.as_str())));
            let value = endpoint.query(&client)?;
            println!("{}", value);
        }
    }

    Ok(())
}

#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
enum CargoCli {
    Api(ApiArgs),
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct ApiArgs {
    #[command(flatten)]
    manifest: clap_cargo::Manifest,

    #[arg(long)]
    user_agent: String,

    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
#[command(propagate_version = true)]
pub enum Subcommand {
    /// Print details for a specific crate.
    ///
    /// Endpoint: /api/v1/crates/:name
    Crate(CrateOpts),
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Crate options")]
pub struct CrateOpts {
    #[arg(value_name = "name")]
    name: String,
}
