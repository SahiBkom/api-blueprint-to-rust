use ::api_blueprint_to_rust::ApibToRs;
use log::*;

fn main() {
    env_logger::init();
    info!("starting up");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        print!("Wrong number of args, need only the apib file");
        return;
    }

    match ApibToRs::new(args[1].to_string()).rs_as_string() {
        Ok(t) => println!("{}", t),
        Err(e) => println!("Error: {}", e),
    }
}
