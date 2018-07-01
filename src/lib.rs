extern crate bodyparser;
#[macro_use] extern crate exonum;
#[macro_use] extern crate failure;
extern crate iron;
extern crate router;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;


pub mod constants;
pub mod schema;
pub mod api;
pub mod wallet;
pub mod errors;
pub mod transactions;


pub mod service {
    use exonum::{api::Api,
                 blockchain::{ApiContext, Service, Transaction, TransactionSet},
                 crypto::Hash,
                 encoding,
                 messages::RawTransaction,
                 storage::Snapshot};
    use iron::Handler;
    use router::Router;

    use constants::{SERVICE_NAME, SERVICE_ID};
    use api::VotesApi;
    use transactions::Transactions;

    pub struct VotesService;

    impl Service for VotesService {
        fn service_name(&self) -> &'static str { SERVICE_NAME }

        fn service_id(&self) -> u16 { SERVICE_ID }

        fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
            let tx = Transactions::tx_from_raw(raw)?;
            Ok(tx.into())
        }

        fn state_hash(&self, _: &Snapshot) -> Vec<Hash> {
            vec![]
        }

        fn public_api_handler(&self, ctx: &ApiContext) -> Option<Box<Handler>> {
            let mut router = Router::new();
            let api = VotesApi::new(ctx.node_channel().clone(), ctx.blockchain().clone());
            api.wire(&mut router);
            Some(Box::new(router))
        }
    }
}
