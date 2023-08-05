use crate::types::{ConfigHr, ListKind, ListSort, TokenUri};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

#[cw_serde]
pub struct InstantiateMsg {
    pub share_fee: Coin,
    pub save_fee: Coin,
}

#[cw_serde]
pub enum ExecuteMsg {
    ChangeConfig {
        config: ConfigHr,
    },

    Message {
        class_id: String,
        message: String,
        meta: Option<Binary>,
    },

    Rate {
        class_id: String,
        v: u8,
    },

    Share {
        class_id: String,
        token: TokenUri,
        chain_id: Option<String>,
    },

    Save {
        class_id: String,
    },

    Unsave {
        class_id: String,
    },

    // Only Admin:
    RemoveMessage {
        id: u64,
    },
    Withdraw {
        receiver: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<crate::types::Nft>)]
    GetList { kind: ListKind, sort: ListSort },

    #[returns(Vec<crate::types::Nft>)]
    GetCurrentNft {},

    #[returns(Vec<crate::types::Nft>)]
    GetNftByIndex { index: u32 },

    #[returns(Vec<crate::types::Nft>)]
    GetNftByClassId { class_id: String },

    #[returns(Vec<crate::types::Nft>)]
    GetUserNftSaved { addr: Addr },

    #[returns(Option<crate::types::Rate>)]
    GetUserNftRate { addr: Addr, class_id: String },

    #[returns(bool)]
    UserHasSavedNft { addr: Addr, class_id: String },

    #[returns(Vec<crate::types::RateCounts>)]
    GetNftRate { class_id: String },

    #[returns(Vec<crate::types::Message>)]
    GetAllMessages {
        from_index: Option<u64>,
        limit: Option<u64>,
    },

    #[returns(Vec<crate::types::Message>)]
    GetNftMessages { class_id: String },

    #[returns(Vec<crate::types::UserStats>)]
    GetUser { addr: Addr },

    #[returns(crate::types::Config)]
    GetConfig {},

    #[returns(crate::types::TotalStats)]
    GetTotalStats {},

    #[returns(String)]
    GetClassId {
        contract_addr: String,
        token_id: String,
    },
}
