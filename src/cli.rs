use clap::{Arg, Command};

pub fn get_command() -> Command<'static> {
    Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("Generate git trophy for contributors")
        .arg(
            Arg::new("repository")
                .help("git repository file path")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::new("year")
                .help("year")
                .takes_value(true)
                .required(false)
        )
}
