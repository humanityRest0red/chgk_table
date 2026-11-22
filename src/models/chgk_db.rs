use calamine::{open_workbook, Data, Reader, Xlsx};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;
use std::path::Path;

use super::chgk_game::Player;

// ---------- Игрок ----------

#[derive(Clone)]
pub struct PlayerRecord {
    pub last_name: String,
    pub first_name: String,
    pub middle_name: String,
    pub team_code: String, // колонка A
    pub team_name: String, // колонка B
    pub role: char,        // колонка D: Б / К / Л
    pub id: usize,         // колонка E
}

impl PlayerRecord {
    pub fn full_name(&self) -> String {
        let mut s = self.last_name.clone();
        s.push(' ');
        s.push_str(&self.first_name);
        if !self.middle_name.is_empty() {
            s.push(' ');
            s.push_str(&self.middle_name);
        }
        s
    }

    pub fn role_for(&self, team_code: &str, team_name: &str) -> String {
        let matches = if !team_code.is_empty() {
            self.team_code == team_code
        } else if !team_name.is_empty() {
            normalize(&self.team_name) == normalize(team_name)
        } else {
            false
        };

        if matches {
            "Б".to_string()
        } else {
            "Л".to_string()
        }
    }

    pub fn to_player_for_team(&self, team_code: &str, team_name: &str) -> Player {
        Player {
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            middle_name: self.middle_name.clone(),
            id: self.id,
            role: self.role_for(team_code, team_name),
        }
    }
}

// ---------- Команда ----------

#[derive(Clone)]
pub struct TeamRecord {
    pub code: String,
    pub name: String,
}

// ---------- База ----------

#[derive(Clone, Default)]
pub struct ChgkDatabase {
    pub players: Vec<PlayerRecord>,
    pub teams: Vec<TeamRecord>,
}

fn normalize(s: &str) -> String {
    s.to_lowercase().replace('ё', "е")
}

impl ChgkDatabase {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let mut wb: Xlsx<_> = open_workbook(path).map_err(std::io::Error::other)?;

        let sheet = wb
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| std::io::Error::other("пустая книга"))?;

        let range = wb.worksheet_range(&sheet).map_err(std::io::Error::other)?;

        // Универсальное чтение ячейки: строки, числа и булы → текст
        let cell = |row: &[calamine::Data], i: usize| -> String {
            match row.get(i) {
                Some(Data::String(s)) => s.trim().to_string(),
                Some(Data::Int(n)) => n.to_string(),
                Some(Data::Float(f)) => {
                    if f.fract() == 0.0 {
                        (*f as i64).to_string() // Excel хранит целые как 99270.0
                    } else {
                        format!("{f}")
                    }
                }
                Some(Data::Bool(b)) => b.to_string(),
                _ => String::new(),
            }
        };

        let num_cell =
            |row: &[calamine::Data], i: usize| -> usize { cell(row, i).parse().unwrap_or(0) };

        let players: Vec<PlayerRecord> = range
            .rows()
            .skip(1)
            .filter(|row| row.len() >= 8)
            .filter(|row| !cell(row, 5).is_empty())
            .map(|row| PlayerRecord {
                team_code: cell(row, 0),
                team_name: cell(row, 1),
                role: cell(row, 3).chars().next().unwrap_or(' '),
                id: num_cell(row, 4),
                last_name: cell(row, 5),
                first_name: cell(row, 6),
                middle_name: cell(row, 7),
            })
            .collect();

        // Уникальные команды в порядке появления в базе
        let mut teams: Vec<TeamRecord> = Vec::new();
        let mut seen: HashMap<(String, String), ()> = HashMap::new();
        for p in &players {
            if p.team_name.is_empty() {
                continue;
            }
            let key = (p.team_code.clone(), p.team_name.clone());
            if seen.insert(key, ()).is_none() {
                teams.push(TeamRecord {
                    code: p.team_code.clone(),
                    name: p.team_name.clone(),
                });
            }
        }

        Ok(Self { players, teams })
    }

    /// Возвращает до `limit` подсказок: сначала совпадения по началу фамилии,
    /// затем подстрока, затем нечёткое совпадение по полному ФИО.
    pub fn search(&self, query: &str, limit: usize) -> Vec<PlayerRecord> {
        let matcher = SkimMatcherV2::default().ignore_case();
        let q = normalize(query.trim());
        if q.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(i64, &PlayerRecord)> = self
            .players
            .iter()
            .filter_map(|rec| {
                let last = normalize(&rec.last_name);

                let mut score = 0i64;
                if last.starts_with(&q) {
                    score = 200;
                } else if last.contains(&q) {
                    score = 100;
                }
                if let Some(fz) = matcher.fuzzy_match(&normalize(&rec.full_name()), &q) {
                    score = score.max(fz);
                }

                (score > 0).then_some((score, rec))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.truncate(limit);
        scored.into_iter().map(|(_, rec)| rec.clone()).collect()
    }

    /// Подсказки по названию команды: до `limit` пар (code, name).
    pub fn search_team(&self, query: &str, limit: usize) -> Vec<(String, String)> {
        let matcher = SkimMatcherV2::default().ignore_case();
        let q = normalize(query.trim());
        if q.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(i64, &TeamRecord)> = self
            .teams
            .iter()
            .filter_map(|team| {
                let name = normalize(&team.name);

                let mut score = 0i64;
                if name == q {
                    score = 300;
                } else if name.starts_with(&q) {
                    score = 200;
                } else if name.contains(&q) {
                    score = 100;
                }
                if let Some(fz) = matcher.fuzzy_match(&name, &q) {
                    score = score.max(fz);
                }

                (score > 0).then_some((score, team))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.truncate(limit);
        scored
            .into_iter()
            .map(|(_, t)| (t.code.clone(), t.name.clone()))
            .collect()
    }

    /// Ищет команду по названию: точное совпадение → начало → подстрока.
    /// Возвращает (team_code, team_name).
    pub fn find_team(&self, query: &str) -> Option<(String, String)> {
        self.search_team(query, 1)
            .into_iter()
            .next()
            .filter(|(code, name)| {
                // не отдаём результат нечёткого «угадывания», если в запросе
                // нет ни точного совпадения, ни начала названия
                let q = normalize(query.trim());
                let n = normalize(name);
                n == q || n.starts_with(&q) || code.is_empty() == false
            })
    }

    pub fn roster_of_team(&self, code: &str, name: &str) -> Vec<Player> {
        let norm = normalize(name);
        self.players
            .iter()
            .filter(|p| {
                if !code.is_empty() {
                    p.team_code == code
                } else {
                    !norm.is_empty() && normalize(&p.team_name) == norm
                }
            })
            .map(|p| p.to_player_for_team(code, name))
            .collect()
    }
}

#[derive(Default)]
pub struct RosterSearchState {
    pub query: String,
    pub suggestions: Vec<PlayerRecord>,
}
