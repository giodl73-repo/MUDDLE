use macroquad::prelude::*;
use muddle_macroquad::{
    default_macroquad_hosts, macroquad_host_list, macroquad_usage, parse_macroquad_run_options,
    MuddleMacroquadMode, MuddleMacroquadPlayLayout, MuddleMacroquadState,
    MuddleMacroquadTextRegion,
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
    let mut command_buttons: Vec<(Rect, usize)> = Vec::new();

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
                MuddleMacroquadMode::SaveSlots => state.select_previous_slot(),
            }
        }
        if is_key_pressed(KeyCode::Down) {
            match state.mode() {
                MuddleMacroquadMode::HostChooser => state.select_next_host(),
                MuddleMacroquadMode::Playing => state.recall_next_command(),
                MuddleMacroquadMode::SaveSlots => state.select_next_slot(),
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
                MuddleMacroquadMode::SaveSlots => {
                    if let Err(error) = state.load_selected_slot() {
                        eprintln!("{error}");
                    }
                }
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
            let result = match state.mode() {
                MuddleMacroquadMode::SaveSlots => state.save_selected_slot(),
                _ => state.save_now(),
            };
            if let Err(error) = result {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F7) {
            if let Err(error) = state.reload_save() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F8) {
            state.open_save_slots();
        }
        if is_key_pressed(KeyCode::F10) {
            if let Err(error) = state.load_selected_slot() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F11) {
            if let Err(error) = state.export_selected_slot_text() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::Delete) {
            if let Err(error) = state.delete_selected_slot() {
                eprintln!("{error}");
            }
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && state.mode() == MuddleMacroquadMode::Playing
        {
            let (x, y) = mouse_position();
            let mouse = Vec2::new(x, y);
            if let Some((_, index)) = command_buttons
                .iter()
                .find(|(rect, _)| rect.contains(mouse))
            {
                state.submit_command_hint(*index);
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            match state.mode() {
                MuddleMacroquadMode::SaveSlots => state.close_save_slots(),
                _ => break,
            }
        }

        command_buttons.clear();
        match state.mode() {
            MuddleMacroquadMode::HostChooser | MuddleMacroquadMode::SaveSlots => {
                draw_lines(&state.display_lines(), 24.0, 28.0)
            }
            MuddleMacroquadMode::Playing => {
                if let Some(layout) = state.play_layout() {
                    draw_play_layout(&layout, &mut command_buttons);
                }
            }
        }

        next_frame().await;
    }
}

fn draw_play_layout(layout: &MuddleMacroquadPlayLayout, command_buttons: &mut Vec<(Rect, usize)>) {
    let margin = 18.0;
    let width = screen_width();
    let height = screen_height();
    draw_region(
        Rect::new(margin, margin, width - margin * 2.0, 92.0),
        "MUDDLE Macroquad",
        &layout.header,
        Color::from_rgba(28, 36, 48, 255),
    );

    let body_top = 122.0;
    let command_height = 94.0;
    let status_height = 92.0;
    let body_height = height - body_top - command_height - status_height - margin * 2.0;
    let left_width = (width - margin * 3.0) * 0.62;
    let right_width = width - margin * 3.0 - left_width;
    draw_region(
        Rect::new(margin, body_top, left_width, body_height),
        &layout.room.label,
        &layout.room.lines,
        Color::from_rgba(20, 27, 36, 255),
    );

    let right = Rect::new(
        margin * 2.0 + left_width,
        body_top,
        right_width,
        body_height,
    );
    draw_panel_stack(right, &layout.panels);

    let command_rect = Rect::new(
        margin,
        height - command_height - status_height - margin,
        width - margin * 2.0,
        command_height,
    );
    draw_command_buttons(command_rect, &layout.commands, command_buttons);

    let status_rect = Rect::new(
        margin,
        height - status_height,
        (width - margin * 3.0) * 0.5,
        status_height - margin,
    );
    draw_region(
        status_rect,
        &layout.status.label,
        &layout.status.lines,
        Color::from_rgba(28, 36, 48, 255),
    );
    let history_rect = Rect::new(
        status_rect.x + status_rect.w + margin,
        status_rect.y,
        width - status_rect.w - margin * 3.0,
        status_rect.h,
    );
    draw_region(
        history_rect,
        &layout.history.label,
        &layout.history.lines,
        Color::from_rgba(20, 27, 36, 255),
    );
}

fn draw_panel_stack(rect: Rect, panels: &[MuddleMacroquadTextRegion]) {
    if panels.is_empty() {
        draw_region(
            rect,
            "Panels",
            &["No host panels available.".to_string()],
            Color::from_rgba(20, 27, 36, 255),
        );
        return;
    }

    let gap = 8.0;
    let panel_height =
        ((rect.h - gap * (panels.len().saturating_sub(1) as f32)) / panels.len() as f32).max(60.0);
    let mut y = rect.y;
    for panel in panels {
        draw_region(
            Rect::new(rect.x, y, rect.w, panel_height),
            &panel.label,
            &panel.lines,
            Color::from_rgba(20, 27, 36, 255),
        );
        y += panel_height + gap;
        if y > rect.y + rect.h {
            break;
        }
    }
}

fn draw_command_buttons(
    rect: Rect,
    commands: &[muddle_macroquad::MuddleMacroquadCommandControl],
    command_buttons: &mut Vec<(Rect, usize)>,
) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(28, 36, 48, 255),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::from_rgba(82, 100, 128, 255),
    );
    draw_text("Commands", rect.x + 12.0, rect.y + 24.0, 20.0, WHITE);
    if commands.is_empty() {
        draw_text(
            "No command hints from this host.",
            rect.x + 12.0,
            rect.y + 54.0,
            18.0,
            GRAY,
        );
        return;
    }

    let mut x = rect.x + 12.0;
    let mut y = rect.y + 38.0;
    for command in commands {
        let label = command.command.as_str();
        let text_size = measure_text(label, None, 18, 1.0);
        let button_width = (text_size.width + 26.0).min(rect.w - 24.0);
        if x + button_width > rect.x + rect.w - 12.0 {
            x = rect.x + 12.0;
            y += 32.0;
        }
        if y + 28.0 > rect.y + rect.h - 8.0 {
            break;
        }
        let button = Rect::new(x, y, button_width, 26.0);
        draw_rectangle(
            button.x,
            button.y,
            button.w,
            button.h,
            Color::from_rgba(56, 75, 108, 255),
        );
        draw_rectangle_lines(button.x, button.y, button.w, button.h, 1.0, SKYBLUE);
        draw_text(label, button.x + 10.0, button.y + 18.0, 18.0, WHITE);
        command_buttons.push((button, command.index));
        x += button_width + 8.0;
    }
}

fn draw_region(rect: Rect, title: &str, lines: &[String], background: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, background);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::from_rgba(82, 100, 128, 255),
    );
    draw_text(title, rect.x + 12.0, rect.y + 24.0, 20.0, WHITE);
    draw_lines(lines, rect.x + 12.0, rect.y + 50.0);
}

fn draw_lines(lines: &[String], x: f32, mut y: f32) {
    for line in lines {
        for wrapped in wrap_line(line, 92) {
            draw_text(&wrapped, x, y, 18.0, LIGHTGRAY);
            y += 22.0;
            if y > screen_height() - 12.0 {
                return;
            }
        }
        if line.is_empty() {
            y += 8.0;
        }
    }
}

fn wrap_line(line: &str, max_chars: usize) -> Vec<String> {
    if line.len() <= max_chars {
        return vec![line.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        if !current.is_empty() && current.len() + word.len() + 1 > max_chars {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
