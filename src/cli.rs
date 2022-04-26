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
                .required(true),
        )
        .arg(
            Arg::new("year")
                .help("selected year (if not provided sum out every years)")
                .long("year")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("names")
                .help("filter commit with committer name")
                .long("names")
                .takes_value(true)
                .multiple_values(true)
                .required(false),
        )
}
