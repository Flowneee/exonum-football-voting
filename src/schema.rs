use constants::SERVICE_NAME;
use exonum::{crypto::PublicKey,
             storage::{Fork, ProofMapIndex, Snapshot}};
use wallet::*;


pub struct VotesSchema<T> {
    view: T,
}


impl<T: AsRef<Snapshot>> VotesSchema<T> {
    pub fn new(view: T) -> Self {
        VotesSchema { view }
    }

    pub fn fan_wallets(&self) -> ProofMapIndex<&Snapshot, PublicKey, FanWallet> {
        ProofMapIndex::new(format!("{}.{}", SERVICE_NAME, "fan_wallets"),
                           self.view.as_ref())
    }

    pub fn fan_wallet(&self, pub_key: &PublicKey) -> Option<FanWallet> {
        self.fan_wallets().get(pub_key)
    }

    pub fn team_wallets(&self) -> ProofMapIndex<&Snapshot, PublicKey, TeamWallet> {
        ProofMapIndex::new(format!("{}.{}", SERVICE_NAME, "team_wallets"),
                           self.view.as_ref())
    }

    pub fn team_wallet(&self, pub_key: &PublicKey) -> Option<TeamWallet> {
        self.team_wallets().get(pub_key)
    }
}


impl<'a> VotesSchema<&'a mut Fork> {
    pub fn fan_wallets_mut(&mut self)
                           -> ProofMapIndex<&mut Fork, PublicKey, FanWallet> {
        ProofMapIndex::new(format!("{}.{}", SERVICE_NAME, "fan_wallets"),
                           &mut self.view)
    }

    pub fn team_wallets_mut(&mut self)
                           -> ProofMapIndex<&mut Fork, PublicKey, TeamWallet> {
        ProofMapIndex::new(format!("{}.{}", SERVICE_NAME, "team_wallets"),
                           &mut self.view)
    }
}
