use std::{env::args, io, path::PathBuf};
use tracing::error;
use waysight::{backend, state::CONFIG, UserData, USER_DATA};

fn print_usage() {
    let usage_str = "Waysight, the insightful wayland compositor

Usage:
    waysight [ [-a] [--arg] [-k=<value>] [--key=<value>] [-ab -c=<value>] ]

Options:
    -h        --help            Outputs the usage of the waysight command

    -c=value  --config=value    Sets the config file that waysight will be reading from.
                                Defalts to $XDG_CONFIG_HOME/waysight/waysight.toml

    -b=value  --backend=value   Sets the type of backend for waysight to run.
                                Available values are \"drm\" and \"winit\"
                                Will automatically choose backend if option isn't set";
    println!("{}", usage_str);
}

// Parses command-line arguments made by the use
fn parse_args(args: Vec<String>, data: &mut UserData) {
    for arg in &args {
        // Splits key-value pair args. The Some() arm represents key-value pair args. The None arm
        // represents args without a value attached to them
        match arg.split_once("=") {
            Some((key, value)) => match (key, value) {
                ("--config" | "-c", config_path) => {
                    data.config_path = Some(PathBuf::from(config_path));
                }
                _ => {
                    print_usage();
                    return;
                }
            },
            None => {
                if arg.starts_with("--") {
                    match arg.as_str() {
                        "--help" => print_usage(),
                        _ => print_usage(),
                    }
                } else if arg.starts_with('-') {
                    arg.chars().for_each(|flag| match flag {
                        '-' => {}
                        'h' => print_usage(),
                        _ => {
                            error!("Unknown flag");
                            print_usage();
                        }
                    })
                }
            }
        }
    }
}

fn main() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_env("WAYSIGHT_LOGLEVEL") {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(io::stdout)
            .init();
    } else {
        tracing_subscriber::fmt().with_writer(io::stdout).init();
    };
    let mut args: Vec<String> = args().collect();
    args.remove(0);
    let mut mutex_data = USER_DATA.lock().unwrap();
    // Saves speed by not parsing a 0 length argument vec
    if args.len() != 0 {
        parse_args(args, &mut mutex_data);
    }
    drop(mutex_data);
    backend::backend_autoinit();
}
