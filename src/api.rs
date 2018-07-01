use std::str::FromStr;
use std::iter::{IntoIterator};


use bodyparser;
use exonum::{api::{Api, ApiError},
             blockchain::{Blockchain, Transaction},
             crypto::{Hash, PublicKey},
             encoding::serialize::FromHex,
             node::{ApiSender, TransactionSend},
             explorer::{BlockchainExplorer, TransactionInfo::{Committed}}};
use iron::{headers::ContentType, modifiers::Header, prelude::*, status::Status};
use router::Router;
use serde_json;


use schema::*;
use wallet::*;
use transactions::*;


#[derive(Clone)]
pub struct VotesApi {
    channel: ApiSender,
    blockchain: Blockchain,
}


impl VotesApi {
    pub fn new(channel: ApiSender, blockchain: Blockchain) -> VotesApi {
        VotesApi {
            channel,
            blockchain,
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: Hash,
}


impl VotesApi {
    fn post_transaction(&self, req: &mut Request) -> IronResult<Response> {
        match req.get::<bodyparser::Struct<Transactions>>() {
            Ok(Some(transaction)) => {
                let transaction: Box<Transaction> = transaction.into();
                let tx_hash = transaction.hash();
                self.channel.send(transaction).map_err(ApiError::from)?;
                let json = TransactionResponse { tx_hash };
                self.ok_response(&serde_json::to_value(&json).unwrap())
            }
            Ok(None) => Err(ApiError::BadRequest("Empty request body".into()))?,
            Err(e) => Err(ApiError::BadRequest(e.to_string()))?,
        }
    }

    fn get_fan_wallets(&self, _: &mut Request) -> IronResult<Response> {
        let snapshot = self.blockchain.snapshot();
        let schema = VotesSchema::new(snapshot);
        let idx = schema.fan_wallets();
        let wallets: Vec<FanWallet> = idx.values().collect();
        self.ok_response(&serde_json::to_value(&wallets).unwrap())
    }

    fn get_team_wallets(&self, _: &mut Request) -> IronResult<Response> {
        let snapshot = self.blockchain.snapshot();
        let schema = VotesSchema::new(snapshot);
        let idx = schema.team_wallets();
        let wallets: Vec<TeamWallet> = idx.values().collect();
        self.ok_response(&serde_json::to_value(&wallets).unwrap())
    }

    fn get_fan_wallet(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path();
        let wallet_key = path.last().unwrap();
        let public_key = PublicKey::from_hex(wallet_key).map_err(|e| {
            IronError::new(
                e,
                (
                    Status::BadRequest,
                    Header(ContentType::json()),
                    "\"Invalid request param: `pub_key`\"",
                ),
            )
        })?;
        let snapshot = self.blockchain.snapshot();
        let schema = VotesSchema::new(snapshot);
        if let Some(wallet) = schema.fan_wallet(&public_key) {
            self.ok_response(&serde_json::to_value(wallet).unwrap())
        } else {
            self.not_found_response(&serde_json::to_value("Fan wallet not found").unwrap())
        }
    }

    fn get_team_wallet(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path();
        let wallet_key = path.last().unwrap();
        let public_key = PublicKey::from_hex(wallet_key).map_err(|e| {
            IronError::new(
                e,
                (
                    Status::BadRequest,
                    Header(ContentType::json()),
                    "\"Invalid request param: `pub_key`\"",
                ),
            )
        })?;
        let snapshot = self.blockchain.snapshot();
        let schema = VotesSchema::new(snapshot);
        if let Some(wallet) = schema.team_wallet(&public_key) {
            self.ok_response(&serde_json::to_value(wallet).unwrap())
        } else {
            self.not_found_response(&serde_json::to_value("Team wallet not found").unwrap())
        }
    }

    fn get_rating(&self, _: &mut Request) -> IronResult<Response> {
        let snapshot = self.blockchain.snapshot();
        let schema = VotesSchema::new(snapshot);
        let team_wallets = schema.team_wallets();
        let mut teams_vec: Vec<TeamWallet> = team_wallets.into_iter().map(|x| { x.1 }).collect();
        teams_vec.sort_by(|l, r| { r.votes().cmp(&l.votes()) });
        self.ok_response(&serde_json::to_value(teams_vec).unwrap())
    }

    fn get_block_by_fan_vote(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path();
        let wallet_key = path.last().unwrap();
        let public_key = PublicKey::from_hex(wallet_key).map_err(|e| {
            IronError::new(
                e,
                (
                    Status::BadRequest,
                    Header(ContentType::json()),
                    "\"Invalid request param: `pub_key`\"",
                ),
            )
        })?;
        let snapshot = self.blockchain.snapshot();
        let schema = VotesSchema::new(snapshot);
        let fan_wallet = match schema.fan_wallet(&public_key) {
            Some(x) => x,
            None => return self.not_found_response(&serde_json::to_value("Wallet not found").unwrap())
        };
        if !fan_wallet.voted() {
            return self.not_found_response(&serde_json::to_value("Fan not voted yet").unwrap());
        };
        let blockchain_explorer = BlockchainExplorer::new(&self.blockchain);
        let tx_hash = Hash::from_str(fan_wallet.vote_hash()).map_err(|e| {
            IronError::new(
                e,
                (
                    Status::BadRequest,
                    Header(ContentType::json()),
                    "\"Cannot convert string to hash\"",
                ),
            )
        })?;
        if let Some(info) = blockchain_explorer.transaction(&tx_hash) {
            let tx_info = match info {
                Committed(x) => x,
                _ => return self.not_found_response(
                    &serde_json::to_value("Transaction not yet committed").unwrap()
                )
            };
            let block_height = tx_info.location().block_height();
            self.ok_response(&serde_json::to_value(blockchain_explorer.block(block_height).unwrap().header()).unwrap())
        } else {
            self.not_found_response(&serde_json::to_value("Transaction not found").unwrap())
        }
    }
}


impl Api for VotesApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let post_create_wallet = move |req: &mut Request| self_.post_transaction(req);
        let self_ = self.clone();
        let post_vote = move |req: &mut Request| self_.post_transaction(req);
        let self_ = self.clone();
        let get_fan_wallets = move |req: &mut Request| self_.get_fan_wallets(req);
        let self_ = self.clone();
        let get_fan_wallet = move |req: &mut Request| self_.get_fan_wallet(req);
        let self_ = self.clone();
        let get_team_wallets = move |req: &mut Request| self_.get_team_wallets(req);
        let self_ = self.clone();
        let get_team_wallet = move |req: &mut Request| self_.get_team_wallet(req);
        let self_ = self.clone();
        let get_rating = move |req: &mut Request| self_.get_rating(req);
        let self_ = self.clone();
        let get_block = move |req: &mut Request| self_.get_block_by_fan_vote(req);

        router.post("/v1/create", post_create_wallet, "post_create_wallet");
        router.post("/v1/vote", post_vote, "post_vote");
        router.get("/v1/fan/wallets", get_fan_wallets, "get_fan_wallets");
        router.get("/v1/fan/wallet/:pub_key", get_fan_wallet, "get_fan_wallet");
        router.get("/v1/team/wallets", get_team_wallets, "get_team_wallets");
        router.get("/v1/team/wallet/:pub_key", get_team_wallet, "get_team_wallet");
        router.get("/v1/rating", get_rating, "get_rating");
        router.get("/v1/block/:pub_key", get_block, "get_block");
    }
}
