use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bmad-converter")]
#[command(about = "Converts BMAD agent .md files to Rust code for the Pulse plugin")]
struct Args {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(args) {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    use bmad_converter_lib::{codegen, parser};

    let agents = parser::parse_directory(&args.input)?;
    let count = agents.len();
    codegen::write_agent_files(&agents, &args.output)?;
    println!("Processed {} agents → {}", count, args.output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn main_compiles() {}
}
