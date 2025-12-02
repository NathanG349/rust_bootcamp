use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "A friendly command-line greeter.", long_about = None)]
struct Args {

    #[arg(default_value = "World", value_name = "TARGET_NAME")]
    name: String,

    #[arg(short = 'U', long)]
    upper: bool,

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
