use uuid::Uuid;

use crate::models::{CoinBalance, Transaction, TransactionKind};

/// Award coins for completing a habit. Base earn is 1 coin + streak bonus.
pub fn on_complete(coins: &mut CoinBalance, streak_days: u32) {
    let base_earn = 1u32;
    let streak_bonus = streak_bonus(streak_days);
    let total_earn = base_earn + streak_bonus;

    earn(
        coins,
        total_earn,
        format!("Habit complete (streak {}d)", streak_days),
    );
}

/// Calculate streak bonus: every 7 consecutive days grants +1 bonus coin.
fn streak_bonus(streak_days: u32) -> u32 {
    if streak_days < 7 {
        0
    } else {
        (streak_days / 7) as u32
    }
}

/// Add coins to balance and record an immutable transaction.
pub fn earn(coins: &mut CoinBalance, amount: u32, note: String) {
    let new_balance = coins.balance + amount as i64;
    coins.transactions.push(Transaction {
        id: Uuid::new_v4(),
        kind: TransactionKind::Earn(amount),
        amount: amount as i64,
        balance_after: new_balance,
        note,
    });
    coins.balance = new_balance;
}

/// Spend coins (e.g., for a slot spin). Returns true if successful.
pub fn spend(coins: &mut CoinBalance, amount: u32, note: String) -> bool {
    if coins.balance < amount as i64 {
        return false;
    }

    let new_balance = coins.balance - amount as i64;
    coins.transactions.push(Transaction {
        id: Uuid::new_v4(),
        kind: TransactionKind::Spend(amount),
        amount: -(amount as i64),
        balance_after: new_balance,
        note,
    });
    coins.balance = new_balance;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_coins() -> CoinBalance {
        CoinBalance {
            balance: 0,
            transactions: vec![],
        }
    }

    #[test]
    fn earn_increases_balance_and_logs_transaction() {
        let mut coins = fresh_coins();
        earn(&mut coins, 5, "Test earn".to_string());

        assert_eq!(coins.balance, 5);
        assert_eq!(coins.transactions.len(), 1);
        assert_eq!(coins.transactions[0].kind, TransactionKind::Earn(5));
        assert_eq!(coins.transactions[0].balance_after, 5);
    }

    #[test]
    fn earn_multiple_times_accumulates() {
        let mut coins = fresh_coins();
        earn(&mut coins, 3, "First".to_string());
        earn(&mut coins, 2, "Second".to_string());

        assert_eq!(coins.balance, 5);
        assert_eq!(coins.transactions.len(), 2);
        assert_eq!(coins.transactions[0].balance_after, 3);
        assert_eq!(coins.transactions[1].balance_after, 5);
    }

    #[test]
    fn spend_decreases_balance() {
        let mut coins = fresh_coins();
        earn(&mut coins, 10, "Setup".to_string());
        let success = spend(&mut coins, 3, "Spin".to_string());

        assert!(success);
        assert_eq!(coins.balance, 7);
        assert_eq!(coins.transactions.len(), 2);
    }

    #[test]
    fn spend_rejects_overdraw() {
        let mut coins = fresh_coins();
        earn(&mut coins, 3, "Setup".to_string());
        let success = spend(&mut coins, 5, "Too much".to_string());

        assert!(!success);
        assert_eq!(coins.balance, 3); // unchanged
        assert_eq!(coins.transactions.len(), 1); // no new transaction
    }

    #[test]
    fn streak_bonus_formula() {
        assert_eq!(streak_bonus(0), 0);
        assert_eq!(streak_bonus(6), 0);
        assert_eq!(streak_bonus(7), 1);
        assert_eq!(streak_bonus(13), 1);
        assert_eq!(streak_bonus(14), 2);
        assert_eq!(streak_bonus(21), 3);
    }

    #[test]
    fn on_complete_base_plus_bonus() {
        let mut coins = fresh_coins();

        // No streak bonus yet
        on_complete(&mut coins, 0);
        assert_eq!(coins.balance, 1); // base earn only

        // At 7 days streak: 1 + 1 bonus = 2
        on_complete(&mut coins, 7);
        assert_eq!(coins.balance, 3);

        // At 14 days streak: 1 + 2 bonus = 3
        on_complete(&mut coins, 14);
        assert_eq!(coins.balance, 6);
    }

    #[test]
    fn transaction_log_immutability() {
        let mut coins = fresh_coins();
        earn(&mut coins, 5, "Earn".to_string());
        spend(&mut coins, 2, "Spend".to_string());

        // Verify the first transaction wasn't modified by the second operation
        assert_eq!(coins.transactions[0].balance_after, 5);
        assert_eq!(coins.transactions[1].balance_after, 3);
    }

    #[test]
    fn balance_consistency_mixed_operations() {
        let mut coins = fresh_coins();

        earn(&mut coins, 10, "Earn 10".to_string());
        spend(&mut coins, 3, "Spend 3".to_string());
        earn(&mut coins, 5, "Earn 5".to_string());
        spend(&mut coins, 7, "Spend 7".to_string());

        // Final: 10 - 3 + 5 - 7 = 5
        assert_eq!(coins.balance, 5);

        // Verify last transaction's balance_after matches actual balance
        let last_tx = coins.transactions.last().unwrap();
        assert_eq!(last_tx.balance_after, coins.balance);
    }
}
