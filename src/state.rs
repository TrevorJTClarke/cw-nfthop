use std::collections::HashMap;

use crate::types::{Config, Message, Nft, Rate, RateCount, RateCounts, TotalStats, UserStats};
use cosmwasm_std::Addr;
use cw_storage_plus::{Deque, Item, Map};

pub const CONFIG: Item<Config> = Item::new("c");
pub const STATS: Item<TotalStats> = Item::new("s");

// User stuffz
pub const USER_STATS: Map<Addr, UserStats> = Map::new("u");
pub const USER_SAVED: Map<Addr, Vec<String>> = Map::new("us");

// Linked list for nfts
pub const LIST: Deque<String> = Deque::new("n");
pub const NFTS: Map<String, Nft> = Map::new("nft");

// All messages index based on timstamp + tx index
pub const MESSAGES: Map<u64, Message> = Map::new("m");
// A simple cache of messages for individiual NFTs ("class_id")
pub const MESSAGES_IDS: Map<String, Vec<u64>> = Map::new("mi");

// Ratings:
// Timeframes: Day, Month, Year - compute SMA for 24-365 points per NFT
// Average driven by per-NFT 1-6 value, compute: total ratings, avg of all ratings
pub const NFT_RATINGS: Map<(String, Addr), Rate> = Map::new("nr");
pub const NFT_RATE_COUNTS: Map<String, RateCounts> = Map::new("nrc");

// Only store map of rated items, with forced position & truncated
pub const NFT_RATE_ATH: Map<String, RateCount> = Map::new("rath");
pub const NFT_RATE_ATL: Map<String, RateCount> = Map::new("ratl");

// keeps 365 days history of day level, so query can collate sums later
// while this is a naive approach for accuracy, it smooths the larger picture of performances. #HACKIT
pub const NFT_RATE_DAY_ATH: Map<u64, HashMap<String, RateCount>> = Map::new("rdath");
pub const NFT_RATE_DAY_ATL: Map<u64, HashMap<String, RateCount>> = Map::new("rdatl");
