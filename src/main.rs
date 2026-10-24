mod client;
mod crypto;
mod server;

use std::path::PathBuf;

use clap::Parser;
use clap_derive::Subcommand;
use directories::ProjectDirs;

fn project_dirs() -> ProjectDirs {
    directories::ProjectDirs::from("", "ThePangel", "Gutun").expect("could not find home dir")
}

fn local_data() -> PathBuf {
    project_dirs().data_local_dir().to_path_buf()
}

fn server_ip() -> String {
    String::from("test")
}
#[derive(Subcommand, Debug)]
enum Command {
    Pair {
        code: String,
    },
    Enroll {
        #[arg(long, default_value = "0.0.0.0:8080")]
        listen: String,
    },

    Server {
        #[arg(long, default_value = "0.0.0.0:8080")]
        listen: String,
        #[arg(long, default_value_os_t = local_data())]
        data: PathBuf,
    },

    Send {
        file: PathBuf,
        #[arg(long, default_value_os_t = server_ip())]
        server: String,
    },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::Send { file, server } => {
            client::start_client().await?;
        }
        Command::Server { listen, data } => {
            server::start_server(listen).await?;
        }
        Command::Enroll { listen } => server::enroll(listen).await?,
        Command::Pair { code } => client::pair(code).await?,
    }

    Ok(())
}
