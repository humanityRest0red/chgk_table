use crate::models::chgk_game::{Game, Answer};

use chrono::{Datelike, Local};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};
use std::path::{Path, PathBuf};

pub fn export_to_excel(game: Option<Game>) {
    let Some(game) = &game else {
        eprintln!("Нет активной игры для экспорта");
        return;
    };

    let dir = match get_or_create_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Ошибка при создании папки: {}", e);
            return;
        }
    };

    export_results_to_excel(game, &dir);
    export_rosters_to_excel(game, &dir);
}

fn export_results_to_excel(game: &Game, dir: &Path) {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.set_column_width(0, 10.0).unwrap(); // Team ID
    worksheet.set_column_width(1, 20.0).unwrap(); // Name
    worksheet.set_column_width(2, 15.0).unwrap(); // City
    worksheet.set_column_width(3, 8.0).unwrap();  // Round

    let fmt_ok = Format::new()
        .set_font_color(Color::RGB(0x008000)) // зелёный
        .set_align(FormatAlign::Center);

    let fmt_bad = Format::new()
        .set_font_color(Color::Red)
        .set_align(FormatAlign::Center);

    let fmt_question = Format::new()
        .set_font_color(Color::RGB(0x785000)) // тёмно-жёлтый текст
        .set_background_color(Color::Yellow)
        .set_align(FormatAlign::Center);

    let q_per_round = game.questions_per_round();
    let mut current_row = 1;

    for r_idx in 0..game.rounds.len() {
        worksheet.write(current_row, 0, "Team ID").unwrap();
        worksheet.write(current_row, 1, "Название").unwrap();
        worksheet.write(current_row, 2, "Город").unwrap();
        worksheet.write(current_row, 3, "Тур").unwrap();

        for q_offset in 0..q_per_round {
            let col = (4 + q_offset) as u16;
            worksheet.set_column_width(col, 4.0).unwrap();

            let global_q_num = (r_idx * q_per_round) + q_offset + 1;
            worksheet
                .write(current_row, col, global_q_num.to_string())
                .unwrap();
        }

        current_row += 1;

        for (t_idx, team) in game.teams.iter().enumerate() {
            match team.id.trim().parse::<u32>() {
                Ok(n) => { worksheet.write(current_row, 0, n).unwrap(); }
                Err(_) => { worksheet.write(current_row, 0, (t_idx + 1) as u32).unwrap(); }
            }
            worksheet.write(current_row, 1, &team.name).unwrap();
            worksheet.write(current_row, 2, "Магас").unwrap();
            worksheet.write(current_row, 3, (r_idx + 1) as u32).unwrap();

            for q_idx in 0..q_per_round {
                let answer = game.answer(r_idx, q_idx, t_idx);
                let col = (4 + q_idx) as u16;

                let (text, fmt) = match answer {
                    Answer::Plus => ("1", &fmt_ok),
                    Answer::Minus => ("0", &fmt_bad),
                    Answer::Question => ("?", &fmt_question),
                    Answer::Empty => ("", &fmt_ok),
                };

                worksheet
                    .write_with_format(current_row, col, text, fmt)
                    .unwrap();
            }
            current_row += 1;
        }

        current_row += 1;
    }

    let save_path = dir.join("результаты.xlsx");
    if let Err(e) = workbook.save(&save_path) {
        eprintln!("Ошибка при сохранении Excel: {}", e);
    }
}

fn export_rosters_to_excel(game: &Game, dir: &Path) {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Составы").unwrap();

    sheet.set_column_width(0, 8.0).unwrap();   // Team ID
    sheet.set_column_width(1, 20.0).unwrap();  // Название
    sheet.set_column_width(2, 10.0).unwrap();  // Город
    sheet.set_column_width(3, 4.0).unwrap();   // К/Б/Л
    sheet.set_column_width(4, 8.0).unwrap();   // номер
    sheet.set_column_width(5, 20.0).unwrap();  // фамилия
    sheet.set_column_width(6, 15.0).unwrap();  // имя
    sheet.set_column_width(7, 22.0).unwrap();  // отчество

    // Выравнивание: короткие столбцы — по центру, ФИО — влево
    let al_center = Format::new().set_align(FormatAlign::Center);
    let al_left = Format::new().set_align(FormatAlign::Left);

    let border_center = Format::new()
        .set_align(FormatAlign::Center)
        .set_border_top(FormatBorder::Thin);
    let border_left = Format::new()
        .set_align(FormatAlign::Left)
        .set_border_top(FormatBorder::Thin);

    let mut row: u32 = 0;

    for team in &game.teams {
        for (p_idx, player) in team.roster.iter().enumerate() {
            // Первый игрок команды → формат с линией, остальные → обычный
            let (f_center, f_left) = if p_idx == 0 {
                (&border_center, &border_left)
            } else {
                (&al_center, &al_left)
            };

            match team.id.trim().parse::<u32>() {
                Ok(n) => sheet.write_with_format(row, 0, n, f_center).unwrap(),
                Err(_) => sheet.write_with_format(row, 0, &team.id, f_center).unwrap(),
            };

            sheet.write_with_format(row, 1, &team.name, f_left).unwrap();
            sheet.write_with_format(row, 2, "Магас", f_center).unwrap();
            sheet.write_with_format(row, 3, &player.role, f_center).unwrap();
            sheet.write_with_format(row, 4, player.id.to_string(), f_center).unwrap();
            sheet.write_with_format(row, 5, &player.last_name, f_left).unwrap();
            sheet.write_with_format(row, 6, &player.first_name, f_left).unwrap();
            sheet.write_with_format(row, 7, &player.middle_name, f_left).unwrap();

            row += 1;
        }
    }

    if let Err(e) = workbook.save(dir.join("составы.xlsx")) {
        eprintln!("Ошибка при сохранении составов: {}", e);
    }
}


fn get_or_create_dir() -> Result<PathBuf, std::io::Error> {
    let now = Local::now();
    let dir_name = format!("{:02}_{:02}", now.month(), now.day());
    let dir = PathBuf::from("data").join(&dir_name);
    std::fs::create_dir_all(&dir)?; // не падает, если папка уже есть
    Ok(dir)
}

use screenshots::Screen;

pub fn save_region_screenshot(
    area: eframe::egui::Rect,
    ctx: &eframe::egui::Context,
) -> Result<(), Box<dyn std::error::Error>> {
    let ppp = ctx.pixels_per_point();

    let win_pos = ctx
        .input(|i| i.viewport().inner_rect)
        .map(|r| r.min)
        .ok_or("Не удалось получить позицию окна")?;

    // Абсолютные экранные координаты области в физических пикселях
    let x = ((win_pos.x + area.left()) * ppp) as i32;
    let y = ((win_pos.y + area.top()) * ppp) as i32;
    let w = (area.width() * ppp) as u32;
    let h = (area.height() * ppp) as u32;

    // Находим монитор по точке и снимаем нужный прямоугольник
    let screen = Screen::from_point(x, y)?;
    let crop = screen.capture_area(x, y, w, h)?;

    let dir = get_or_create_dir()?;
    let file_name = format!("{}.png", Local::now().format("%H%M%S_%3f"));

    crop.save(dir.join(file_name))?;
    Ok(())
}
