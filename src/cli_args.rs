use structopt::StructOpt;

pub fn config() -> CLIArgs {
    CLIArgs::from_args()
}

#[derive(StructOpt, Debug)]
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
        short = "d",
        long = "dry-run",
        help = "In dryrun mode minos creates the scenarios but does not execute them against the server."
    )]
    pub dry_run: bool,

    #[structopt(
        short = "amrs",
        long = "allow-missing-rs",
        help = "Do not fail the test if the response body do not have a schema defining it. Useful if the API does not document the application error responses."
    )]
    pub allow_missing_rs: bool,

    // TODO: this to do something, right now hardcoded to only passing
    #[structopt(
        short = "op",
        long = "only-passing",
        help = "Only run scenarios that expect success response codes."
    )]
    pub only_passing_scenarios: bool,


    #[structopt(
        short = "hi",
        long = "hammer-it",
        help = "run scenarios as performance suite."
    )]
    pub hammer_it: bool,

    #[structopt(
        short = "t",
        long = "threads",
        help = "Number of users(threads) running the performance suite."
    )]
    pub users: usize,

    #[structopt(
        short = "m",
        long = "matches",
        help = "Only run on paths matching certain paths.",
        default_value = "/"
    )]
    pub matches: String,

    #[structopt(
        short = "s",
        long = "server",
        help = "Command to use to launch server",
        default_value = "bundle exec rails server"
    )]
    pub server_command: String,

    #[structopt(
        short = "it",
        long = "timeout",
        help = "Timeout allowed for the service to startup",
        default_value = "10"
    )]
    pub server_wait: u64,

    #[structopt(long = "run_server", help = "Runs the server itself or not")]
    pub server_run: bool,
}
