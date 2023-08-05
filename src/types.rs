use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin};

// defaults
pub const DEFAULT_UNLOCK_MESSAGES: u64 = 5;
pub const DEFAULT_UNLOCK_GRAFFITI: u64 = 25;
pub const DEFAULT_UNLOCK_SHARES: u64 = 50;
pub const DEFAULT_USER_MAX_SHARES: u64 = 50;
pub const DEFAULT_RATE_DECAY: u64 = 2959200; // 3 days in seconds

pub const MAX_LEN_MESSAGE: usize = 141;
pub const MAX_LEN_ALL_TIME: usize = 100;
pub const MAX_LEN_DAY: usize = 10;
pub const DAY_IN_SECONDS: u64 = 986400; // 1 day in seconds

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub share_fee: Coin,
    pub save_fee: Coin,

    // UI unlocks
    pub unlock_messages: Option<u64>,
    pub unlock_graffiti: Option<u64>,
    pub unlock_share: Option<u64>,
    pub max_shares: Option<u64>,
    pub rate_decay: Option<u64>,
}

#[cw_serde]
pub struct ConfigHr {
    pub owner: Option<Addr>,
    pub share_fee: Option<Coin>,
    pub save_fee: Option<Coin>,

    // UI unlocks
    pub unlock_messages: Option<u64>,
    pub unlock_graffiti: Option<u64>,
    pub unlock_share: Option<u64>,
    pub max_shares: Option<u64>,
    pub rate_decay: Option<u64>,
}

#[cw_serde]
pub struct TotalStats {
    pub nfts: u64,
    pub ratings: u64,
    pub messages: u64,
    pub saves: u64,
}

#[cw_serde]
pub struct UserStats {
    pub last_rate_ts: u64,
    pub ratings: u64,
    pub saves: u64,
    pub shares: u64,
}

#[cw_serde]
pub struct TokenUri {
    /// NFT based metadata URI supported
    pub contract_addr: Addr,
    /// NFT Token ID -- Example: 8394
    pub id: String,
    /// Example: ipfs://bafybeibmhfvddexxwwt52mknwqa74fp7jkqiktedmruxlnuvtbpwbfyhqa/metadata/8394
    pub data_uri: Option<String>,
}

#[cw_serde]
pub struct Nft {
    pub token: TokenUri,

    /// NFT Class ID -- Example: stars1234...abcd_8394
    pub class_id: String,

    /// NFT Chain ID -- Example: stargaze-1, ethereum, optimism, juno-1
    pub chain_id: Option<String>,

    /// The queue place
    pub index: Option<u64>,
}

#[cw_serde]
pub struct Message {
    pub ts: u64,
    pub class_id: String,
    pub message: String,
    pub from: Addr,
    pub meta: Option<Binary>,
}

#[cw_serde]
pub struct Rate {
    pub ts: u64,
    pub v: u8,
}

// sum: the SUM of rates
// Total: count of included sums
// Ts: (optional) keep track for windowed sums, will be the timestamp of the first modulo timstamp past previous start.
#[cw_serde]
pub struct RateCount {
    pub ts: u64,
    pub sum: u64,
    pub total: u64,
}

#[cw_serde]
pub struct RateCounts {
    pub all: RateCount,
    pub day: RateCount,
}

#[cw_serde]
pub enum ListSort {
    Highest,
    Lowest,
}

#[cw_serde]
pub enum ListKind {
    All,
    Day,
    Month,
}
