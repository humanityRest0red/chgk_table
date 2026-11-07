use serde::{Deserialize, Serialize};
use std::cmp::Reverse;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum Answer {
    Plus,
    Minus,
    Question,
    #[default]
    Empty,
}

impl Answer {
    pub fn score(self) -> usize {
        match self {
            Answer::Plus => 1,
            _ => 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Player {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Team {
    pub name: String,
    pub roster: Vec<Player>,
}

impl Team {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            roster: Vec::new(),
        }
    }
}

/// Ответы всех команд на один вопрос.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Question {
    pub answers: Vec<Answer>,
}

impl Question {
    pub fn new(team_count: usize) -> Self {
        Self {
            answers: vec![Answer::Empty; team_count],
        }
    }

    pub fn answer_of(&self, team_index: usize) -> Answer {
        self.answers.get(team_index).copied().unwrap_or(Answer::Empty)
    }

    pub fn set_answer(&mut self, team_index: usize, answer: Answer) {
        if let Some(slot) = self.answers.get_mut(team_index) {
            *slot = answer;
        }
    }

    pub fn plus_count(&self) -> usize {
        self.answers.iter().filter(|&&a| a == Answer::Plus).count()
    }

    /// Рейтинг вопроса: чем меньше команд ответили "+", тем он выше.
    pub fn rating_for(&self, team_count: usize) -> usize {
        team_count.saturating_sub(self.plus_count())
    }

    /// Переставляет ответы команд в новом порядке:
    /// `reorder_answers(&[2, 0, 1])` — старая третья команда становится первой.
    pub fn reorder_answers(&mut self, order: &[usize]) {
        self.answers = order.iter().map(|&old| self.answers[old]).collect();
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Round {
    pub questions: Vec<Question>,
}

impl Round {
    pub fn new(questions_per_round: usize, team_count: usize) -> Self {
        Self {
            questions: (0..questions_per_round)
                .map(|_| Question::new(team_count))
                .collect(),
        }
    }

    pub fn team_score(&self, team_index: usize) -> usize {
        self.questions
            .iter()
            .filter(|q| q.answer_of(team_index) == Answer::Plus)
            .count()
    }

    pub fn team_disputed(&self, team_index: usize) -> usize {
        self.questions
            .iter()
            .filter(|q| q.answer_of(team_index) == Answer::Question)
            .count()
    }

    /// Сумма рейтингов вопросов, на которые команда ответила "+".
    pub fn team_rating(&self, team_index: usize, team_count: usize) -> usize {
        self.questions
            .iter()
            .filter(|q| q.answer_of(team_index) == Answer::Plus)
            .map(|q| q.rating_for(team_count))
            .sum()
    }

    /// Рейтинги всех вопросов раунда.
    pub fn question_ratings(&self, team_count: usize) -> Vec<usize> {
        self.questions
            .iter()
            .map(|q| q.rating_for(team_count))
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Game {
    pub teams: Vec<Team>,
    pub rounds: Vec<Round>,
}

impl Game {
    pub fn new(round_count: usize, questions_per_round: usize, team_count: usize) -> Self {
        Self {
            teams: (1..=team_count)
                .map(|n| Team::new(format!("Команда {n}")))
                .collect(),
            rounds: (0..round_count)
                .map(|_| Round::new(questions_per_round, team_count))
                .collect(),
        }
    }

    pub fn team_count(&self) -> usize {
        self.teams.len()
    }

    pub fn round_count(&self) -> usize {
        self.rounds.len()
    }

    pub fn questions_per_round(&self) -> usize {
        self.rounds.first().map_or(0, |r| r.questions.len())
    }

    pub fn rounds_iter(&self) -> impl Iterator<Item = &Round> {
        self.rounds.iter()
    }

    // ---- агрегаты по всем раундам ----

    pub fn team_score(&self, team_index: usize) -> usize {
        self.rounds_iter().map(|r| r.team_score(team_index)).sum()
    }

    pub fn team_disputed(&self, team_index: usize) -> usize {
        self.rounds_iter().map(|r| r.team_disputed(team_index)).sum()
    }

    pub fn team_rating(&self, team_index: usize) -> usize {
        self.rounds_iter()
            .map(|r| r.team_rating(team_index, self.team_count()))
            .sum()
    }

    /// Текст счёта: "10" без спорных, "10(+1)" — если есть спорные.
    pub fn team_score_text(&self, team_index: usize) -> String {
        let score = self.team_score(team_index);
        let disputed = self.team_disputed(team_index);

        match disputed {
            0 => score.to_string(),
            _ => format!("{}(+{})", score, disputed),
        }
    }

    pub fn round_score_text(&self, team_index: usize, round_index: usize) -> String {
        let score = self.round_team_score(team_index, round_index);
        let disputed = self.round_team_disputed(team_index, round_index);
        
        match disputed {
            0 => score.to_string(),
            _ => format!("{}(+{})", score, disputed),
        }
    }

    pub fn round_team_score(&self, team_index: usize, round_index: usize) -> usize {
        self.rounds
            .get(round_index)
            .map_or(0, |round| round.team_score(team_index))
    }

    pub fn round_team_disputed(&self, team_index: usize, round_index: usize) -> usize {
        self.rounds
            .get(round_index)
            .map_or(0, |round| round.team_disputed(team_index))
    }

    pub fn question_ratings(&self) -> Vec<Vec<usize>> {
        self.rounds_iter()
            .map(|r| r.question_ratings(self.team_count()))
            .collect()
    }

    pub fn answer(&self, round_index: usize, question_index: usize, team_index: usize) -> Answer {
        self.rounds
            .get(round_index)
            .and_then(|r| r.questions.get(question_index))
            .map_or(Answer::Empty, |q| q.answer_of(team_index))
    }

    pub fn set_answer(
        &mut self,
        round_index: usize,
        question_index: usize,
        team_index: usize,
        answer: Answer,
    ) {
        if let Some(q) = self
            .rounds
            .get_mut(round_index)
            .and_then(|r| r.questions.get_mut(question_index))
        {
            q.set_answer(team_index, answer);
        }
    }

    /// Сортирует команды по убыванию рейтинга, при равенстве — по счёту.
    pub fn sort_game_teams(&mut self) {
        let mut order: Vec<usize> = (0..self.team_count()).collect();

        order.sort_by_key(|&i| {
            (Reverse(self.team_rating(i)), Reverse(self.team_score(i)))
        });

        let old_teams = self.teams.clone();
        for (new_index, &old_index) in order.iter().enumerate() {
            self.teams[new_index] = old_teams[old_index].clone();
        }

        for round in self.rounds.iter_mut() {
            for question in round.questions.iter_mut() {
                question.reorder_answers(&order);
            }
        }
    }

    pub fn answer_mut(
        &mut self,
        round_index: usize,
        question_index: usize,
        team_index: usize,
    ) -> &mut Answer {
        self.rounds
            .get_mut(round_index)
            .and_then(|r| r.questions.get_mut(question_index))
            .and_then(|q| q.answers.get_mut(team_index))
            .expect("индекс вне таблицы ответов")
    }
}
