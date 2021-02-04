//use structopt::StructOpt;
use clap::Clap;

pub fn config() -> CLIArgs {
    CLIArgs::parse()
}

impl CLIArgs {
    pub fn generator_config(&self) -> daedalus::GeneratorConfig {
        daedalus::GeneratorConfig::new(
            self.filename.clone(),
            self.conv_filename.clone(),
            self.scenarios_all_codes,
            self.matches.clone(),
        )
    }
}

#[derive(Clap)]
#[clap(
    name = "Minos",
    about = "Minos tests that your OpenAPI file matches your live API."
)]
pub struct CLIArgs {
    #[clap(
        short = 'f',
        long = "file",
        about = "Input OpenAPI file",
        default_value = "doc/contracts/openapi.yaml"
    )]
    pub filename: String,

    #[clap(
        short = 'u',
        long = "url",
        about = "URL where the server is running (it can also be in localhost)",
        default_value = "http://localhost:3000"
    )]
    pub base_url: String,

    #[clap(
        short = 'c',
        long = "conversions",
        about = "The location of the conversions file with parameter values for this run.",
        default_value = "./conversions.yml"
    )]
    pub conv_filename: String,

    #[clap(
        short = 'm',
        long = "matches",
        about = "Only generate scenarios for paths matching certain expression.",
        default_value = "/"
    )]
    pub matches: String,

    #[clap(
        short = 'a',
        long = "all-codes",
        about = "Generate scenarios for all codes. Default is to generate only scenarios with 200 codes."
    )]
    pub scenarios_all_codes: bool,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Clap)]
pub enum Command {
    #[clap(about = "Shows generated scenarios but does not run them.")]
    Ls,

    Performance(PerformanceCommand),

    #[clap(
        about = "Runs auto-generated scenarios to verify the service follows the openapi contract"
    )]
    Verify {
        #[clap(
            short = 'n',
            long = "allow-missing-rs",
            about = "Do not fail the test if the response body do not have a schema defining it. Useful if the API does not document the application error responses."
        )]
        without_rs: bool,
    },
}

#[derive(Clap)]
#[clap(about = "Runs auto-genenerated scenarios as performance tests.")]
pub struct PerformanceCommand {
    #[clap(
        short = 't',
        long = "threads",
        about = "Number of threads(users) running the performance suite. More users more resources used.",
        default_value = "16"
    )]
    pub users: usize,
    #[clap(
        short = 'r',
        long = "requests",
        about = "Number of maximum requests per second to run against the whole service.",
        default_value = "100"
    )]
    pub request_per_second: usize,

    #[clap(
        short = 'l',
        long = "length",
        about = "Length of test, as readable time (300s, 20m, 3h, 1h30m, etc.) .",
        default_value = "90s"
    )]
    pub time: String,
}
