use macroquad::prelude::*;
use muddle_macroquad::MuddleMacroquadState;

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
    let mut state = MuddleMacroquadState::new().expect("MUDDLE macroquad state starts");

    loop {
        clear_background(Color::from_rgba(18, 23, 30, 255));

        while let Some(character) = get_char_pressed() {
            state.push_char(character);
        }
        if is_key_pressed(KeyCode::Backspace) {
            state.backspace();
        }
        if is_key_pressed(KeyCode::Enter) {
            state.submit_input();
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
