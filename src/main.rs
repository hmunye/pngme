//! Command-line tool for embedding and retrieving messages in PNG files.

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

mod chunk;
mod chunk_type;
mod commands;
mod png;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "pngme")]
#[command(version, about = None, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Commands {
    /// Encodes a message into a PNG file.
    #[command(arg_required_else_help = true)]
    Encode {
        #[command(flatten)]
        opts: CommandOpts,
        #[arg(value_name = "message")]
        message: String,
        #[arg(value_name = "outfile")]
        output_path: Option<PathBuf>,
    },
    /// Decodes a message from a PNG file.
    #[command(arg_required_else_help = true)]
    Decode {
        #[command(flatten)]
        opts: CommandOpts,
    },
    /// Removes a message from a PNG file.
    #[command(arg_required_else_help = true)]
    Remove {
        #[command(flatten)]
        opts: CommandOpts,
        #[arg(value_name = "outfile")]
        output_path: Option<PathBuf>,
    },
}

#[derive(Args, Debug)]
struct CommandOpts {
    #[arg(value_name = "infile")]
    file_path: PathBuf,
    #[arg(value_name = "chunk_type")]
    chunk_type: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Encode {
            opts,
            message,
            output_path,
        } => {
            commands::invoke_encode(opts.file_path, opts.chunk_type, message, output_path)?;
        }
        Commands::Decode { opts } => {
            if let Some(message) = commands::invoke_decode(opts.file_path, opts.chunk_type)? {
                println!("{message}");
            }
        }
        Commands::Remove { opts, output_path } => {
            if let Some(message) =
                commands::invoke_remove(opts.file_path, opts.chunk_type, output_path)?
            {
                println!("{message}");
            }
        }
    }

    Ok(())
}
