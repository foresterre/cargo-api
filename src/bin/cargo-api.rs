use cargo_api::client::ReqwestClient;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let CargoCli::Api(args) = CargoCli::parse();
    println!("{:?}", args.user_agent);

    let _client = ReqwestClient::new(args.user_agent.as_str());

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
}
