use macroquad::prelude::*;
use muddle_macroquad::{
    default_macroquad_hosts, macroquad_window_conf, parse_macroquad_run_options,
    run_muddle_macroquad_hosts, MuddleMacroquadRunConfig,
};

fn window_conf() -> Conf {
    macroquad_window_conf("MUDDLE Macroquad Runner")
}

#[macroquad::main(window_conf)]
async fn main() {
    let options = match parse_macroquad_run_options(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", muddle_macroquad::macroquad_usage());
            std::process::exit(1);
        }
    };

    if let Err(error) = run_muddle_macroquad_hosts(
        default_macroquad_hosts(),
        options,
        MuddleMacroquadRunConfig::default(),
    )
    .await
    {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
