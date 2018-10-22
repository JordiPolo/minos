use structopt::StructOpt; 

pub fn config() -> CLIArgs {
    CLIArgs::from_args()
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Minos", about = "Minos tests that your OpenAPI file matches your live API.")]
pub struct CLIArgs {
    /// Needed parameter, the first on the command line.
    #[structopt(short = "f", help = "Input OpenAPI file", default_value = "doc/contracts/openapi.yaml")]
    pub filename: String,

    /// Needed parameter, the first on the command line.
    #[structopt(short = "b", help = "URL where the server is running (it can also be in localhost)", default_value = "http://localhost:3000")]
    pub base_url: String,

    #[structopt(short = "s", help = "Command to use to launch server", default_value = "bundle exec rails server")]
    pub server_command: String,

    #[structopt(short = "t", help = "Timeout allowed for the service to startup", default_value = "10")]
    pub server_wait: u64,

  //  #[structopt(short = "rs", help = "Runs the server itself or not", default_value = "true")]
  //  pub server_run: bool,
}
