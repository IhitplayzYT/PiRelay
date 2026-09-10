use std::fs;

use crate::helper::Helper::CLI;

mod PiRelay;
mod helper;



#[tokio::main]
async fn main() {
    let mut clargs = CLI::new();
    clargs.Parse_Args();
    if clargs.dbg{
        println!("{:?}",clargs)
    }
    let mut conf_json = "".to_string();
    if let Some(x) = &clargs.config{
        conf_json += &fs::read_to_string(&x).unwrap();
    }

    if !conf_json.is_empty() {
        if let Err(e) = PiRelay::send_config(&clargs, &conf_json).await {
            eprintln!("Failed to send config: {}", e);
        }
    }

    let result = match clargs.method.as_str() {
        "SEND" => PiRelay::send(&clargs).await,
        "RECEIVE" => PiRelay::receive(&clargs).await,
        _ => {
            eprintln!("Unknown method: {}", clargs.method);
            helper::Helper::Help();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error executing command: {}", e);
    }
}

