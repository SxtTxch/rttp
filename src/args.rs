use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub ip: String,
    #[arg(short, long)]
    pub port: String,
}

pub fn get_args() -> Args {
    Args::parse()
}
