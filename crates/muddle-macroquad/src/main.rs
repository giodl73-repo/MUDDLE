use macroquad::prelude::*;
use muddle_macroquad::{
    default_macroquad_hosts, macroquad_host_list, macroquad_usage, parse_macroquad_run_options,
    MuddleMacroquadMode, MuddleMacroquadState,
};

fn window_conf() -> Conf {
    Conf {
        window_title: "MUDDLE Macroquad Runner".to_string(),
        window_width: 960,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let registrations = default_macroquad_hosts();
    let options = match parse_macroquad_run_options(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", macroquad_usage());
            return;
        }
    };
    if options.show_help {
        println!("{}", macroquad_usage());
        println!("{}", macroquad_host_list(&registrations));
        return;
    }
    if options.list_hosts {
        println!("{}", macroquad_host_list(&registrations));
        return;
    }

    let mut state = match options.host_name {
        Some(host_name) => MuddleMacroquadState::with_host_and_paths(
            registrations,
            &host_name,
            options.load_path,
            options.save_path,
            options.transcript_path,
        ),
        None => MuddleMacroquadState::with_chooser_and_paths(
            registrations,
            options.load_path,
            options.save_path,
            options.transcript_path,
        ),
    }
    .expect("MUDDLE macroquad state starts");

    loop {
        clear_background(Color::from_rgba(18, 23, 30, 255));

        while let Some(character) = get_char_pressed() {
            state.push_char(character);
        }
        if is_key_pressed(KeyCode::Backspace) {
            state.backspace();
        }
        if is_key_pressed(KeyCode::Up) {
            match state.mode() {
                MuddleMacroquadMode::HostChooser => state.select_previous_host(),
                MuddleMacroquadMode::Playing => state.recall_previous_command(),
            }
        }
        if is_key_pressed(KeyCode::Down) {
            match state.mode() {
                MuddleMacroquadMode::HostChooser => state.select_next_host(),
                MuddleMacroquadMode::Playing => state.recall_next_command(),
            }
        }
        if is_key_pressed(KeyCode::Enter) {
            match state.mode() {
                MuddleMacroquadMode::HostChooser => {
                    if let Err(error) = state.choose_selected_host() {
                        eprintln!("{error}");
                    }
                }
                MuddleMacroquadMode::Playing => state.submit_input(),
            }
        }
        if is_key_pressed(KeyCode::F2) {
            state.change_host();
        }
        if is_key_pressed(KeyCode::F5) {
            if let Err(error) = state.restart_host() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F6) {
            if let Err(error) = state.save_now() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F7) {
            if let Err(error) = state.reload_save() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        let mut y = 28.0;
        for line in state.display_lines() {
            draw_text(&line, 24.0, y, 22.0, LIGHTGRAY);
            y += if line.is_empty() { 12.0 } else { 26.0 };
            if y > screen_height() - 20.0 {
                break;
            }
        }

        next_frame().await;
    }
}
