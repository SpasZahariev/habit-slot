//! SQLite persistence layer.
//! Schema migrations, CRUD for all tables, graceful degradation on errors.

use chrono::{Datelike, NaiveDate};
#[cfg(feature = "db")]
use rusqlite::Connection;
use std::collections::HashSet;
use uuid::Uuid;

use crate::models::{CoinBalance, Completion, Habit, RewardPool};

/// Current schema version. Increment on every migration.
const SCHEMA_VERSION: i32 = 1;

#[cfg(feature = "db")]
pub struct Db {
    conn: Connection,
}

#[cfg(feature = "db")]
impl Db {
    /// Open or create a database file. Runs migrations if needed.
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Create an in-memory database (for testing).
    #[cfg(test)]
    pub fn open_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open("")?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Initialize schema + seed empty state.
    fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value INTEGER NOT NULL);",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS habits (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at DATE NOT NULL);",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS completions (habit_id TEXT NOT NULL, date DATE NOT NULL, \
             PRIMARY KEY (habit_id, date), FOREIGN KEY (habit_id) REFERENCES habits(id));",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS coin_balance (id INTEGER PRIMARY KEY CHECK (id = 1), balance INTEGER NOT NULL DEFAULT 0);",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transactions (id TEXT PRIMARY KEY, kind TEXT NOT NULL, \
             amount INTEGER NOT NULL, balance_after INTEGER NOT NULL, note TEXT, created_at DATETIME);",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS milestones (\
             habit_id TEXT PRIMARY KEY, \
             claimed_streak_tiers TEXT NOT NULL DEFAULT '{}', \
             claimed_completion_tiers TEXT NOT NULL DEFAULT '{}', \
             FOREIGN KEY (habit_id) REFERENCES habits(id));",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS pity_counter (id INTEGER PRIMARY KEY CHECK (id = 1), consecutive_losses INTEGER NOT NULL DEFAULT 0);",
        )?;

        Self::ensure_version(conn)?;

        conn.execute(
            "INSERT OR IGNORE INTO coin_balance (id, balance) VALUES (1, 0)",
            (),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO pity_counter (id, consecutive_losses) VALUES (1, 0)",
            (),
        )?;

        Ok(())
    }

    fn ensure_version(conn: &Connection) -> Result<(), rusqlite::Error> {
        let current: Option<i32> = conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'schema_version'",
                (),
                |row| row.get(0),
            )
            .ok()
            .flatten();

        if current.map_or(true, |v| v < SCHEMA_VERSION) {
            conn.execute(
                "INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', ?1)",
                [&SCHEMA_VERSION],
            )?;
        }

        Ok(())
    }

    // -- Habits --

    pub fn insert_habit(
        &self,
        id: Uuid,
        name: &str,
        created_at: NaiveDate,
    ) -> Result<(), rusqlite::Error> {
        let date_str = format_date(created_at);
        self.conn.execute(
            "INSERT OR REPLACE INTO habits (id, name, created_at) VALUES (?1, ?2, ?3)",
            [&id.to_string(), name, &date_str],
        )?;

        self.conn.execute(
            "INSERT OR IGNORE INTO milestones (habit_id) VALUES (?1)",
            [&id.to_string()],
        )?;

        Ok(())
    }

    pub fn delete_habit(&self, id: Uuid) -> Result<(), rusqlite::Error> {
        let s = id.to_string();
        self.conn
            .execute("DELETE FROM completions WHERE habit_id = ?1", [&s])?;
        self.conn
            .execute("DELETE FROM milestones WHERE habit_id = ?1", [&s])?;
        self.conn
            .execute("DELETE FROM habits WHERE id = ?1", [&s])?;
        Ok(())
    }

    pub fn load_habits(&self) -> Result<Vec<Habit>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, created_at FROM habits ORDER BY created_at")?;
        let rows = stmt.query_map((), |row| {
            Ok(Habit {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                name: row.get(1)?,
                created_at: NaiveDate::parse_from_str(&row.get::<_, String>(2)?, "%Y-%m-%d")
                    .unwrap_or_default(),
                reward_pool: RewardPool::default(),
            })
        })?;

        rows.collect()
    }

    // -- Completions --

    pub fn insert_completion(
        &self,
        habit_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), rusqlite::Error> {
        let date_str = format_date(date);
        self.conn.execute(
            "INSERT OR IGNORE INTO completions (habit_id, date) VALUES (?1, ?2)",
            [&habit_id.to_string(), &date_str],
        )?;
        Ok(())
    }

    pub fn delete_completion(
        &self,
        habit_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), rusqlite::Error> {
        let date_str = format_date(date);
        self.conn.execute(
            "DELETE FROM completions WHERE habit_id = ?1 AND date = ?2",
            [&habit_id.to_string(), &date_str],
        )?;
        Ok(())
    }

    pub fn load_completions(&self) -> Result<Vec<Completion>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT habit_id, date FROM completions")?;
        let rows = stmt.query_map((), |row| {
            Ok(Completion {
                habit_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                date: NaiveDate::parse_from_str(&row.get::<_, String>(1)?, "%Y-%m-%d")
                    .unwrap_or_default(),
            })
        })?;

        rows.collect()
    }

    // -- Coin balance & transactions --

    pub fn load_coin_balance(&self) -> Result<CoinBalance, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, amount, balance_after, note FROM transactions ORDER BY rowid",
        )?;

        let mut balance = CoinBalance::default();
        for row in stmt.query_map((), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })? {
            let (id_s, kind, amount, balance_after, note) = row?;
            let kind_val = if kind == "earn" {
                crate::models::TransactionKind::Earn(amount.max(0) as u32)
            } else {
                crate::models::TransactionKind::Spend((-amount).max(0) as u32)
            };

            balance.transactions.push(crate::models::Transaction {
                id: Uuid::parse_str(&id_s).unwrap_or_default(),
                kind: kind_val,
                amount,
                balance_after,
                note: note.unwrap_or_default(),
            });
        }

        balance.balance = balance
            .transactions
            .last()
            .map(|t| t.balance_after)
            .unwrap_or(0);
        Ok(balance)
    }

    pub fn insert_transaction(
        &self,
        tx: &crate::models::Transaction,
    ) -> Result<(), rusqlite::Error> {
        let kind_str = match tx.kind {
            crate::models::TransactionKind::Earn(_) => "earn",
            crate::models::TransactionKind::Spend(_) => "spend",
        };

        self.conn.execute(
            "INSERT INTO transactions (id, kind, amount, balance_after, note) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![tx.id.to_string(), kind_str, tx.amount, tx.balance_after, &tx.note],
        )?;

        self.conn.execute(
            "UPDATE coin_balance SET balance = ?1 WHERE id = 1",
            [tx.balance_after],
        )?;

        Ok(())
    }

    // -- Milestones --

    pub fn load_milestone_tracker(
        &self,
        habit_id: Uuid,
    ) -> Result<crate::rewards::MilestoneTracker, rusqlite::Error> {
        let (streak_str, completion_str): (String, String) = self.conn.query_row(
            "SELECT claimed_streak_tiers, claimed_completion_tiers FROM milestones WHERE habit_id = ?1",
            [&habit_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or(("{}".to_string(), "{}".to_string()));

        Ok(crate::rewards::MilestoneTracker {
            claimed_streak_tiers: parse_tier_set(&streak_str),
            claimed_completion_tiers: parse_tier_set(&completion_str),
        })
    }

    pub fn save_milestone_tracker(
        &self,
        habit_id: Uuid,
        tracker: &crate::rewards::MilestoneTracker,
    ) -> Result<(), rusqlite::Error> {
        let streak_str = format_tier_set(&tracker.claimed_streak_tiers);
        let completion_str = format_tier_set(&tracker.claimed_completion_tiers);

        self.conn.execute(
            "UPDATE milestones SET claimed_streak_tiers = ?2, claimed_completion_tiers = ?3 WHERE habit_id = ?1",
            [habit_id.to_string(), streak_str, completion_str],
        )?;
        Ok(())
    }

    // -- Pity counter --

    pub fn load_pity_counter(&self) -> Result<u32, rusqlite::Error> {
        self.conn.query_row(
            "SELECT consecutive_losses FROM pity_counter WHERE id = 1",
            (),
            |row| row.get(0),
        )
    }

    pub fn save_pity_counter(&self, losses: u32) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE pity_counter SET consecutive_losses = ?1 WHERE id = 1",
            [losses],
        )?;
        Ok(())
    }
}

fn format_date(d: NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

/// Parse comma-separated tier indices from storage string.
fn parse_tier_set(s: &str) -> HashSet<usize> {
    s.split(',')
        .filter(|x| !x.is_empty())
        .filter_map(|x| x.parse().ok())
        .collect()
}

/// Format tier set as comma-separated indices for storage.
fn format_tier_set(set: &HashSet<usize>) -> String {
    let mut v: Vec<_> = set.iter().map(|x| x.to_string()).collect();
    v.sort();
    v.join(",")
}

#[cfg(feature = "db")]
#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn schema_creates_all_tables() {
        let _db = Db::open_memory().unwrap();
        // If init_schema failed, this would panic.
    }

    #[test]
    fn habit_crud_roundtrip() {
        let db = Db::open_memory().unwrap();
        let id = Uuid::new_v4();
        let name = "Test Habit".to_string();
        let created = date(2026, 5, 7);

        db.insert_habit(id, &name, created).unwrap();

        let habits = db.load_habits().unwrap();
        assert_eq!(habits.len(), 1);
        assert_eq!(habits[0].id, id);
        assert_eq!(habits[0].name, name);
        assert_eq!(habits[0].created_at, created);

        db.delete_habit(id).unwrap();
        assert!(db.load_habits().unwrap().is_empty());
    }

    #[test]
    fn completion_crud_roundtrip() {
        let db = Db::open_memory().unwrap();
        let habit_id = Uuid::new_v4();
        db.insert_habit(habit_id, "Test", date(2026, 5, 1)).unwrap();

        let comp_date = date(2026, 5, 7);
        db.insert_completion(habit_id, comp_date).unwrap();

        let completions = db.load_completions().unwrap();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].habit_id, habit_id);
        assert_eq!(completions[0].date, comp_date);

        db.delete_completion(habit_id, comp_date).unwrap();
        assert!(db.load_completions().unwrap().is_empty());
    }

    #[test]
    fn coin_balance_persists_across_transactions() {
        let db = Db::open_memory().unwrap();

        let tx = crate::models::Transaction {
            id: Uuid::new_v4(),
            kind: crate::models::TransactionKind::Earn(10),
            amount: 10,
            balance_after: 10,
            note: "test earn".to_string(),
        };
        db.insert_transaction(&tx).unwrap();

        let tx2 = crate::models::Transaction {
            id: Uuid::new_v4(),
            kind: crate::models::TransactionKind::Spend(3),
            amount: -3,
            balance_after: 7,
            note: "test spend".to_string(),
        };
        db.insert_transaction(&tx2).unwrap();

        let balance = db.load_coin_balance().unwrap();
        assert_eq!(balance.balance, 7);
        assert_eq!(balance.transactions.len(), 2);
    }

    #[test]
    fn transaction_log_immutability() {
        let db = Db::open_memory().unwrap();

        let tx = crate::models::Transaction {
            id: Uuid::new_v4(),
            kind: crate::models::TransactionKind::Earn(5),
            amount: 5,
            balance_after: 5,
            note: "first".to_string(),
        };
        db.insert_transaction(&tx).unwrap();

        let loaded1 = db.load_coin_balance().unwrap();
        let loaded2 = db.load_coin_balance().unwrap();

        assert_eq!(loaded1.transactions.len(), loaded2.transactions.len());
        assert_eq!(loaded1.transactions[0].id, loaded2.transactions[0].id);
    }

    #[test]
    fn persistence_survives_close_reopen() {
        let path = "/tmp/habit_slot_test.db";
        let _ = std::fs::remove_file(path);

        {
            let db1 = Db::open(path).unwrap();
            let id = Uuid::new_v4();
            db1.insert_habit(id, "Persisted Habit", date(2026, 5, 7))
                .unwrap();
            db1.insert_completion(id, date(2026, 5, 7)).unwrap();

            let tx = crate::models::Transaction {
                id: Uuid::new_v4(),
                kind: crate::models::TransactionKind::Earn(15),
                amount: 15,
                balance_after: 15,
                note: "persist test".to_string(),
            };
            db1.insert_transaction(&tx).unwrap();
        }

        {
            let db2 = Db::open(path).unwrap();
            let habits = db2.load_habits().unwrap();
            assert_eq!(habits.len(), 1);
            assert_eq!(habits[0].name, "Persisted Habit");

            let completions = db2.load_completions().unwrap();
            assert_eq!(completions.len(), 1);

            let balance = db2.load_coin_balance().unwrap();
            assert_eq!(balance.balance, 15);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn milestone_tracker_crud() {
        let db = Db::open_memory().unwrap();
        let habit_id = Uuid::new_v4();
        db.insert_habit(habit_id, "Test", date(2026, 5, 1)).unwrap();

        let tracker = db.load_milestone_tracker(habit_id).unwrap();
        assert!(tracker.claimed_streak_tiers.is_empty());
        assert!(tracker.claimed_completion_tiers.is_empty());

        let mut tracker2 = tracker.clone();
        tracker2.claimed_streak_tiers.insert(0);
        db.save_milestone_tracker(habit_id, &tracker2).unwrap();

        let tracker3 = db.load_milestone_tracker(habit_id).unwrap();
        assert!(tracker3.claimed_streak_tiers.contains(&0));
        assert!(!tracker3.claimed_streak_tiers.contains(&1));
    }

    #[test]
    fn pity_counter_crud() {
        let db = Db::open_memory().unwrap();

        let losses = db.load_pity_counter().unwrap();
        assert_eq!(losses, 0);

        db.save_pity_counter(3).unwrap();
        assert_eq!(db.load_pity_counter().unwrap(), 3);
    }

    #[test]
    fn schema_migration_sets_version() {
        let db = Db::open_memory().unwrap();
        let version: i32 = db
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'schema_version'",
                (),
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(version, SCHEMA_VERSION);
    }
}
