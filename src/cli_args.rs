use structopt::StructOpt;

pub fn config() -> CLIArgs {
    CLIArgs::from_args()
}

#[derive(StructOpt)]
#[structopt(
    name = "Minos",
    about = "Minos tests that your OpenAPI file matches your live API."
)]
pub struct CLIArgs {
    #[structopt(
        short = "f",
        long = "file",
        help = "Input OpenAPI file",
        default_value = "doc/contracts/openapi.yaml"
    )]
    pub filename: String,

    #[structopt(
        short = "u",
        long = "url",
        help = "URL where the server is running (it can also be in localhost)",
        default_value = "http://localhost:3000"
    )]
    pub base_url: String,

    #[structopt(
        short = "c",
        long = "conversions",
        help = "The location of the conversions file with parameter values for this run.",
        default_value = "./conversions.minos"
    )]
    pub conv_filename: String,

    #[structopt(
        short = "m",
        long = "matches",
        help = "Only generate scenarios for paths matching certain expression.",
        default_value = "/"
    )]
    pub matches: String,

    // TODO: this to do something, right now hardcoded to only 200 code
    #[structopt(
        short = "a",
        long = "all-codes",
        help = "Generate scenarios for all codes. Default is to generate only scenarios with 200 codes."
    )]
    pub scenarios_all_codes: bool,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(about = "Shows generated scenarios but does not run them.")]
    Ls,

    #[structopt(about = "Runs auto-genenerated scenarios as performance tests.")]
    Performance {
        #[structopt(
            short = "t",
            long = "threads",
            help = "Number of threads(users) running the performance suite.",
            default_value = "8"
        )]
        users: usize,
    },

    #[structopt(
        about = "Runs auto-generated scenarios to verify the service follows the openapi contract"
    )]
    Verify {
        #[structopt(
            short = "n",
            long = "allow-missing-rs",
            help = "Do not fail the test if the response body do not have a schema defining it. Useful if the API does not document the application error responses."
        )]
        allow_missing_rs: bool,
    },
}
