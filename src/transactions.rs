use exonum::{blockchain::{ExecutionResult, Transaction},
             messages::Message,
             storage::Fork,
             crypto::{Hash, CryptoHash, PublicKey}};


use constants::SERVICE_ID;
use errors::*;
use schema::*;
use wallet::*;


transactions! {
    pub Transactions {
        const SERVICE_ID = SERVICE_ID;

        struct TxCreateWallet {
            pub_key: &PublicKey,
            name: &str,
            is_team: bool
        }

        struct TxVote {
            from: &PublicKey,
            to: &PublicKey,
            seed: u64,
        }
    }
}


impl Transaction for TxCreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        println!("{:?}", self);
        let mut schema = VotesSchema::new(view);
        if self.is_team() {
            if schema.team_wallet(self.pub_key()).is_none() {
                let wallet = TeamWallet::new(self.pub_key(), self.name(), 0);
                println!("Create the team: {:?}", wallet);
                schema.team_wallets_mut().put(self.pub_key(), wallet);
                Ok(())
            } else {
                Err(Error::WalletAlreadyExists)?
            }
        } else {
            if schema.fan_wallet(self.pub_key()).is_none() {
                let wallet = FanWallet::new(self.pub_key(), self.name(), false,
                                            &Hash::zero().to_hex());
                println!("Create the fan: {:?}", wallet);
                schema.fan_wallets_mut().put(self.pub_key(), wallet);
                Ok(())
            } else {
                Err(Error::WalletAlreadyExists)?
            }
        }
    }
}


impl Transaction for TxVote {
    fn verify(&self) -> bool {
        self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = VotesSchema::new(view);

        let sender = match schema.fan_wallet(self.from()) {
            Some(val) => val,
            None => Err(Error::SenderNotFound)?,
        };

        let receiver = match schema.team_wallet(self.to()) {
            Some(val) => val,
            None => Err(Error::ReceiverNotFound)?,
        };

        if !sender.voted() {
            let sender = sender.vote(self.hash());
            let receiver = receiver.add_vote();
            println!("Vote: {:?} => {:?}", sender, receiver);
            schema.fan_wallets_mut().put(self.from(), sender);
            schema.team_wallets_mut().put(self.to(), receiver);
            Ok(())
        } else {
            Err(Error::InsufficientCurrencyAmount)?
        }
    }
}
