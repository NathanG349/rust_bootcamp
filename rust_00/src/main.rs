use clap::Parser;

/// Simple command-line greeting utility (Bonjour/Hello).
#[derive(Parser, Debug)]
#[command(author, version, about = "A friendly command-line greeter.", long_about = None)]
struct Args {
    /// The name of the person to be greeted. Defaults to "World".
    #[arg(default_value = "World", value_name = "TARGET_NAME")]
    name: String,

    /// Convert the entire greeting message to uppercase.
    #[arg(short = 'U', long)]
    upper: bool,

    /// Specifies how many times the greeting should be repeated (N).
    #[arg(long, default_value_t = 1)]
    repeat: u32,
}

fn main() {
    let args = Args::parse();

    let mut greeting = format!("Hello, {}!", args.name);

    if args.upper {
        greeting = greeting.to_uppercase();
    }

    (0..args.repeat).for_each(|_| println!("{}", greeting));
}
