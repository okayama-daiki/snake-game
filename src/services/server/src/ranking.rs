use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

const LEADERBOARD_SIZE: usize = 10;

#[derive(Clone, Debug, PartialEq)]
struct CurrentScore {
    name: String,
    score: usize,
    is_bot: bool,
    player_token: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RankingEntry {
    pub name: String,
    pub score: usize,
    pub is_bot: bool,
    pub rank: usize,
    pub is_self: bool,
}

#[derive(Default)]
pub struct RankingStore {
    current_scores: HashMap<Uuid, CurrentScore>,
}

pub type SharedRanking = Arc<RwLock<RankingStore>>;

impl RankingStore {
    pub fn update(
        &mut self,
        id: Uuid,
        name: &str,
        score: usize,
        is_bot: bool,
        player_token: Option<Uuid>,
    ) {
        self.current_scores.insert(
            id,
            CurrentScore {
                name: name.to_string(),
                score,
                is_bot,
                player_token,
            },
        );
    }

    pub fn remove(&mut self, id: &Uuid) {
        self.current_scores.remove(id);
    }

    pub fn leaderboard(&self, player_token: Option<Uuid>) -> Vec<RankingEntry> {
        let mut entries: Vec<_> = self.current_scores.values().collect();
        entries.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.name.cmp(&right.name))
        });
        let self_index = player_token.and_then(|token| {
            entries
                .iter()
                .position(|entry| entry.player_token == Some(token))
        });
        let mut visible_indices: Vec<_> = (0..entries.len().min(LEADERBOARD_SIZE)).collect();
        if let Some(index) = self_index.filter(|index| *index >= LEADERBOARD_SIZE) {
            visible_indices.push(index);
        }

        visible_indices
            .into_iter()
            .map(|index| {
                let entry = entries[index];
                RankingEntry {
                    name: entry.name.clone(),
                    score: entry.score,
                    is_bot: entry.is_bot,
                    rank: index + 1,
                    is_self: self_index == Some(index),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_a_players_current_score() {
        let mut ranking = RankingStore::default();
        let id = Uuid::new_v4();
        ranking.update(id, "Alice", 20, false, None);
        ranking.update(id, "Alice", 12, false, None);

        assert_eq!(
            ranking.leaderboard(None),
            vec![RankingEntry {
                name: "Alice".to_string(),
                score: 12,
                is_bot: false,
                rank: 1,
                is_self: false,
            }]
        );
    }

    #[test]
    fn removes_players_that_are_no_longer_playing() {
        let mut ranking = RankingStore::default();
        let id = Uuid::new_v4();
        ranking.update(id, "Alice", 20, false, None);
        ranking.remove(&id);

        assert!(ranking.leaderboard(None).is_empty());
    }

    #[test]
    fn sorts_descending_and_limits_the_result() {
        let mut ranking = RankingStore::default();
        for score in 0..20 {
            ranking.update(
                Uuid::new_v4(),
                &format!("Player {score}"),
                score,
                false,
                None,
            );
        }

        let entries = ranking.leaderboard(None);
        assert_eq!(entries.len(), LEADERBOARD_SIZE);
        assert_eq!(entries[0].score, 19);
        assert_eq!(entries[9].score, 10);
    }

    #[test]
    fn appends_the_current_player_when_outside_the_top_ten() {
        let mut ranking = RankingStore::default();
        let player_token = Uuid::new_v4();
        ranking.update(
            Uuid::new_v4(),
            "Current Player",
            1,
            false,
            Some(player_token),
        );
        for score in 2..=12 {
            ranking.update(Uuid::new_v4(), "Other", score, false, None);
        }

        let entries = ranking.leaderboard(Some(player_token));
        assert_eq!(entries.len(), LEADERBOARD_SIZE + 1);
        assert_eq!(entries.last().unwrap().rank, 12);
        assert!(entries.last().unwrap().is_self);
    }
}
