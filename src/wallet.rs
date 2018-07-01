use exonum::crypto::{Hash, PublicKey};


encoding_struct! {
    struct FanWallet {
        pub_key: &PublicKey,
        name: &str,
        voted: bool,
        vote_hash: &str
    }
}


encoding_struct! {
    struct TeamWallet {
        pub_key: &PublicKey,
        name: &str,
        votes: u64,
    }
}


impl FanWallet {
    pub fn vote(self, vote_tx_hash: Hash) -> Self {
        Self::new(
            self.pub_key(),
            self.name(),
            true,
            &vote_tx_hash.to_hex()
        )
    }
}


impl TeamWallet {
    pub fn add_vote(self) -> Self {
        Self::new(
            self.pub_key(),
            self.name(),
            self.votes() + 1
        )
    }
}
