use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        println!(
            "{}",
            muddle_browser_player::evaluate(&line.expect("UTF-8 request"))
        );
    }
}
