//! Command-line tool for embedding and retrieving messages in PNG files.

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod chunk;
mod chunk_type;
mod png;

use std::path::PathBuf;

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
    /// Encodes a message into the PNG file given its chunk type.
    #[command(arg_required_else_help = true)]
    Encode {
        #[command(flatten)]
        opts: CommandOpts,
        #[arg(value_name = "message")]
        message: String,
    },
    /// Decodes a message from the PNG file given the chunk type.
    #[command(arg_required_else_help = true)]
    Decode {
        #[command(flatten)]
        opts: CommandOpts,
    },
    /// Removes a message from the PNG file given the chunk type.
    #[command(arg_required_else_help = true)]
    Remove {
        #[command(flatten)]
        opts: CommandOpts,
    },
}

#[derive(Args, Debug)]
struct CommandOpts {
    #[arg(value_name = "file_path")]
    file_path: PathBuf,
    #[arg(value_name = "chunk_type")]
    chunk_type: String,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Encode { opts, message } => {
            println!(
                "encoding message '{}' as type '{}' in file {}",
                message,
                opts.chunk_type,
                opts.file_path.display()
            );
        }
        Commands::Decode { opts } => {
            println!(
                "decoding type '{}' from file {}",
                opts.chunk_type,
                opts.file_path.display()
            );
        }
        Commands::Remove { opts } => {
            println!(
                "removing type '{}' from file {}",
                opts.chunk_type,
                opts.file_path.display()
            );
        }
    }
}
