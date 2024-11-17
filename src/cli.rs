use clap::{Arg, Command};

pub fn get_command() -> Command<'static> {
    Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("Generate git trophy for contributors")
        .arg(
            Arg::new("repository")
                .help("git repository file path (if multiple paths are provided then contributions will be summed across the repositories)")
                .takes_value(true)
                .multiple_values(true)
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
                .help("filter commits with committer name")
                .long("names")
                .takes_value(true)
                .multiple_values(true)
                .required(false),
        )
        .arg(
            Arg::new("clip")
                .help("clip commits (limit max number of commits per day)")
                .long("clip")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("ttf_font_path")
                .help("path to ttf font file such as `FiraCode-Medium.ttf`")
                .long("font")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("side_text")
                .help("text to write on the side, please provide the TTF font to use using `--font`")
                .long("text")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("output")
                .help("output file path")
                .long("output")
                .default_value("trophy")
                .takes_value(true)
                .required(false),
        )
}
