use crate::models::chgk_game::Game;

use rust_xlsxwriter::Workbook;

const SAVE_PATH: &'static str = "data/game_results.xlsx";

pub fn export_to_excel(game: Option<Game>) {
    let Some(game) = &game else {
        eprintln!("Нет активной игры для экспорта");
        return;
    };

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.set_column_width(0, 10.0).unwrap(); // Team ID
    worksheet.set_column_width(1, 20.0).unwrap(); // Name
    worksheet.set_column_width(2, 15.0).unwrap(); // City
    worksheet.set_column_width(3, 8.0).unwrap(); // Round

    let q_per_round = game.questions_per_round();
    let mut current_row = 0;

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
            worksheet.write(current_row, 0, (t_idx + 1) as u32).unwrap();
            worksheet.write(current_row, 1, &team.name).unwrap();
            worksheet.write(current_row, 2, "Магас").unwrap();
            worksheet.write(current_row, 3, (r_idx + 1) as u32).unwrap();

            for q_idx in 0..q_per_round {
                let answer = game.answer(r_idx, q_idx, t_idx);
                worksheet
                    .write(current_row, (4 + q_idx) as u16, answer.score() as u16)
                    .unwrap();
            }
            current_row += 1;
        }

        current_row += 1; // Пустая строка между турами, как на картинке
    }

    if let Err(e) = workbook.save(SAVE_PATH) {
        eprintln!("Ошибка при сохранении Excel: {}", e);
    }
}
