// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use exonum::blockchain::{ExecutionError, ExecutionResult, Transaction};
use exonum::crypto::{CryptoHash, PublicKey};
use exonum::messages::Message;
use exonum::storage::Fork;
use chrono::Utc;
use time::Duration;
use chrono::prelude::*;


use CRYPTOCURRENCY_SERVICE_ID;
use schema::CurrencySchema;

/// Error codes emitted by wallet transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Wallet already exists.
    ///
    /// Can be emitted by `CreateWallet`.
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `Transfer` or `Issue`.
    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

transactions! {
    pub WalletTransactions {
        const SERVICE_ID = CRYPTOCURRENCY_SERVICE_ID;

        /// Transfer `amount` of the currency from one wallet to another.
        struct Transfer {
            from:    &PublicKey,
            to:      &PublicKey,
            to_second:      &PublicKey,
            to_third:      &PublicKey,
            amount:  u64,
            seed:    u64,
            // time:   i64,
            tx_time:    &str,
        }

        /// Issue `amount` of the currency to the `wallet`.
        struct Issue {
            pub_key:  &PublicKey,
            amount:  u64,
            amount_second:  u64,
            amount_third:  u64,
            seed:    u64,
        }

        /// Create wallet with the given `name`.
        struct CreateWallet {
            pub_key: &PublicKey,
            name:    &str,
        }
    }
}

impl Transfer {
    fn timestamping(&self) -> i64 
    {
        let now = Utc::now();
        let seconds: i64 = now.timestamp();
        let nanoseconds: i64 = now.nanosecond() as i64;
        (seconds * 1000) + (nanoseconds / 1000 / 1000)
    }
}

impl Transaction for Transfer {
    fn verify(&self) -> bool {
        self.verify_signature(self.from())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CurrencySchema::new(fork);

        let from = self.from();
        let to = self.to();
        let to_second = self.to_second();
        let to_third = self.to_third();
        let hash = self.hash();
        let amount = self.amount();

        // let time = self.time();
        let tx_time_str = self.tx_time();
        let tx_time = tx_time_str.parse::<i64>().unwrap();

        let sender = schema.wallet(from).ok_or(Error::SenderNotFound)?;

        let receiver = schema.wallet(to).ok_or(Error::ReceiverNotFound)?;
        let receiver_second = schema.wallet(to_second).ok_or(Error::ReceiverNotFound)?;
        let receiver_third = schema.wallet(to_third).ok_or(Error::ReceiverNotFound)?;

        if sender.balance() < amount {
            Err(Error::InsufficientCurrencyAmount)?
        }

        let current_time = self.timestamping();

        if current_time > tx_time && current_time < (tx_time + 2160000)
        {
            println!("=========== Success =============");
            println!("=========== Previous time: {} =============", tx_time);
            println!("=========== Current time: {} =============", current_time);
            
            let total_amount = amount * 3;
            schema.decrease_wallet_balance(sender, total_amount, &hash);
            schema.increase_wallet_balance(receiver, amount, &hash);
            schema.increase_wallet_balance(receiver_second, amount, &hash);
            schema.increase_wallet_balance(receiver_third, amount, &hash);
        }
        else
        {
            println!("=========== Failure =============");
            println!("=========== Previous time: {} =============", tx_time);
            println!("=========== Current time: {} =============", current_time);
        }

        // schema.decrease_wallet_balance(sender, amount, &hash);
        // schema.increase_wallet_balance(receiver, amount, &hash);

        Ok(())
    }
}

impl Transaction for Issue {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CurrencySchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if let Some(wallet) = schema.wallet(pub_key) {
            let amount = self.amount();
            let amount_second = self.amount_second();
            let amount_third = self.amount_third();
            let sum_amount = amount_third + amount + amount_second;
            schema.increase_wallet_balance(wallet, sum_amount, &hash);
            Ok(())
        } else {
            Err(Error::ReceiverNotFound)?
        }
    }
}

impl Transaction for CreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CurrencySchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if schema.wallet(pub_key).is_none() {
            let name = self.name();
            schema.create_wallet(pub_key, name, &hash);
            Ok(())
        } else {
            Err(Error::WalletAlreadyExists)?
        }
    }
}
