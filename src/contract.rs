use std::collections::HashMap;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    CONFIG, LIST, MESSAGES, MESSAGES_IDS, NFTS, NFT_RATE_ATH, NFT_RATE_ATL, NFT_RATE_COUNTS,
    NFT_RATE_DAY_ATH, NFT_RATE_DAY_ATL, NFT_RATINGS, STATS, USER_SAVED, USER_STATS,
};
use crate::types::{
    Config, ConfigHr, ListKind, ListSort, Message, Nft, Rate, RateCount, RateCounts, TokenUri,
    TotalStats, UserStats, DAY_IN_SECONDS, DEFAULT_RATE_DECAY, DEFAULT_UNLOCK_GRAFFITI,
    DEFAULT_UNLOCK_MESSAGES, DEFAULT_UNLOCK_SHARES, DEFAULT_USER_MAX_SHARES, MAX_LEN_ALL_TIME,
    MAX_LEN_DAY, MAX_LEN_MESSAGE,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, SubMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

// version info for migration info
const CONTRACT_NAME: &str = "nft-hop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn compare_desc(a: &(String, RateCount), b: &(String, RateCount)) -> std::cmp::Ordering {
    let avg_a: u128 = u128::from(a.1.sum) * u128::from(b.1.total);
    let avg_b: u128 = u128::from(b.1.sum) * u128::from(a.1.total);
    avg_b.cmp(&avg_a)
}
fn compare_asc(a: &(String, RateCount), b: &(String, RateCount)) -> std::cmp::Ordering {
    let avg_a: u128 = u128::from(a.1.sum) * u128::from(b.1.total);
    let avg_b: u128 = u128::from(b.1.sum) * u128::from(a.1.total);
    avg_a.cmp(&avg_b)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = Config {
        owner: info.sender,
        share_fee: msg.share_fee.clone(),
        save_fee: msg.save_fee,

        unlock_messages: Some(DEFAULT_UNLOCK_MESSAGES),
        unlock_graffiti: Some(DEFAULT_UNLOCK_GRAFFITI),
        unlock_share: Some(DEFAULT_UNLOCK_SHARES),
        max_shares: Some(DEFAULT_USER_MAX_SHARES),
        rate_decay: Some(DEFAULT_RATE_DECAY),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &state)?;

    let stats = TotalStats {
        nfts: 0,
        ratings: 0,
        messages: 0,
        saves: 0,
    };
    STATS.save(deps.storage, &stats)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ChangeConfig { config } => change_config(deps, info, config),
        ExecuteMsg::Message {
            class_id,
            message,
            meta,
        } => send_message(deps, env, info, class_id, message, meta),
        ExecuteMsg::Rate { class_id, v } => rate(deps, env, info, class_id, v),
        ExecuteMsg::Share {
            class_id,
            token,
            chain_id,
        } => share(deps, env, info, class_id, token, chain_id),
        ExecuteMsg::Save { class_id } => save(deps, env, info, class_id),
        ExecuteMsg::Unsave { class_id } => unsave(deps, env, info, class_id),
        ExecuteMsg::RemoveMessage { id } => remove_message(deps, info, id),
        ExecuteMsg::Withdraw { receiver } => withdraw(deps, env, info, receiver),
    }
}

// Update configurations
pub fn change_config(
    deps: DepsMut,
    info: MessageInfo,
    config: ConfigHr,
) -> Result<Response, ContractError> {
    let c: Config = CONFIG.load(deps.storage)?;
    // Only owner can do this
    if info.sender != c.owner {
        return Err(ContractError::Unauthorized {});
    }

    let owner_addr = if let Some(owner) = config.owner.clone() {
        deps.api.addr_validate(owner.as_str())?
    } else {
        info.sender.clone()
    };

    CONFIG.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        if let Some(share_fee) = config.share_fee {
            state.share_fee = share_fee;
        }
        if let Some(unlock_messages) = config.unlock_messages {
            state.unlock_messages = Some(unlock_messages);
        }
        if let Some(unlock_graffiti) = config.unlock_graffiti {
            state.unlock_graffiti = Some(unlock_graffiti);
        }
        if let Some(unlock_share) = config.unlock_share {
            state.unlock_share = Some(unlock_share);
        }
        if let Some(rate_decay) = config.rate_decay {
            state.rate_decay = Some(rate_decay);
        }

        // NOTE: Better to do a validated transfer flow, but in this case we're doing naive approach to SHIPPIT
        if owner_addr != info.sender.clone() && config.owner.is_some() {
            state.owner = owner_addr;
        }

        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "change_config"))
}

pub fn rate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    class_id: String,
    v: u8,
) -> Result<Response, ContractError> {
    // check NFT exists
    if !NFTS.has(deps.storage, class_id.clone()) {
        return Err(ContractError::CustomError {
            val: "NFT doesnt exist, cannot rate nothing silly human".to_string(),
        });
    }

    // Check if already rated!
    let rate_key = (class_id.clone(), info.sender.clone());
    if NFT_RATINGS.has(deps.storage, rate_key.clone()) {
        return Err(ContractError::CustomError {
            val: "Already rated this nft, cannot rate again".to_string(),
        });
    }

    // check value is valid (thank you GPT!! LOL)
    if !(1..=5).contains(&v) {
        return Err(ContractError::CustomError {
            val: "Invalid rate value".to_string(),
        });
    }

    let ts = env.block.time.seconds();
    let sender_rate = Rate { ts, v };
    NFT_RATINGS.save(deps.storage, rate_key, &sender_rate)?;

    // get the previous ts offset via remainders
    let day_remainder = ts % DAY_IN_SECONDS;
    let year_remainder = ts % (DAY_IN_SECONDS * 366);
    let last_day_ts = ts.saturating_sub(day_remainder);
    let last_year_ts = ts.saturating_sub(year_remainder);

    // Truncate day caches older than 1 year ago
    let ath_keys = NFT_RATE_DAY_ATH
        .keys(
            deps.storage,
            None,
            Some(Bound::inclusive(last_year_ts)),
            Order::Descending,
        )
        .map(|res| res.unwrap())
        .collect::<Vec<_>>();
    let atl_keys = NFT_RATE_DAY_ATL
        .keys(
            deps.storage,
            None,
            Some(Bound::inclusive(last_year_ts)),
            Order::Descending,
        )
        .map(|res| res.unwrap())
        .collect::<Vec<_>>();

    for hk in ath_keys.into_iter() {
        NFT_RATE_DAY_ATH.remove(deps.storage, hk);
    }
    for lk in atl_keys.into_iter() {
        NFT_RATE_DAY_ATL.remove(deps.storage, lk);
    }

    // compute total rate counts
    let rate_counts = NFT_RATE_COUNTS.update(
        deps.storage,
        class_id.clone(),
        |counts| -> Result<_, ContractError> {
            match counts {
                Some(rc) => {
                    let mut r = rc;
                    // update all counts
                    r.all.ts = 0;
                    r.all.total = r.all.total.saturating_add(1);
                    r.all.sum = r.all.sum.saturating_add(v as u64);

                    if r.day.ts + last_day_ts > ts {
                        // Still within range
                        r.day.ts = ts;
                        r.day.total = r.day.total.saturating_add(1);
                        r.day.sum = r.day.sum.saturating_add(v as u64);
                    } else {
                        // outside range, start fresh
                        r.day.ts = last_day_ts;
                        r.day.total = 1;
                        r.day.sum = v as u64;
                    }
                    Ok(r)
                }
                None => {
                    let default_rc = RateCount {
                        ts,
                        sum: v as u64,
                        total: 1,
                    };
                    Ok(RateCounts {
                        all: default_rc.clone(),
                        day: default_rc,
                    })
                }
            }
        },
    )?;

    // - append to state immediately, since its more efficient than checking if exists first
    // - then tally all rate counts and remove from storage if found out of bounds item(s)
    NFT_RATE_ATH.save(deps.storage, class_id.clone(), &rate_counts.all)?;
    NFT_RATE_ATL.save(deps.storage, class_id.clone(), &rate_counts.all)?;

    let mut all_ath = NFT_RATE_ATH
        .range(deps.storage, None, None, Order::Descending)
        .map(|res| res.unwrap())
        .collect::<Vec<(String, RateCount)>>();
    let mut all_atl = NFT_RATE_ATL
        .range(deps.storage, None, None, Order::Descending)
        .map(|res| res.unwrap())
        .collect::<Vec<(String, RateCount)>>();

    // sort then truncate as needed
    all_ath.sort_by(compare_desc);
    if all_ath.len() > MAX_LEN_ALL_TIME {
        for (k, _) in all_ath[MAX_LEN_ALL_TIME..].iter() {
            NFT_RATE_ATH.remove(deps.storage, k.to_string());
        }
    }
    all_atl.sort_by(compare_asc);
    if all_ath.len() > MAX_LEN_ALL_TIME {
        for (k, _) in all_ath[MAX_LEN_ALL_TIME..].iter() {
            NFT_RATE_ATL.remove(deps.storage, k.to_string());
        }
    }

    // sort days then truncate as needed
    NFT_RATE_DAY_ATH.update(
        deps.storage,
        last_day_ts,
        |rates| -> Result<_, ContractError> {
            match rates {
                Some(s) => {
                    let mut days: Vec<_> = s.into_iter().collect();
                    days.sort_by(compare_desc);
                    if days.len() > MAX_LEN_DAY {
                        days.truncate(MAX_LEN_DAY + 1);
                    }
                    let days_map: HashMap<String, RateCount> = days.into_iter().collect();
                    Ok(days_map)
                }
                None => {
                    let mut n = HashMap::new();
                    n.insert(
                        class_id.clone(),
                        RateCount {
                            ts,
                            sum: v as u64,
                            total: 1,
                        },
                    );
                    Ok(n)
                }
            }
        },
    )?;
    NFT_RATE_DAY_ATL.update(
        deps.storage,
        last_day_ts,
        |rates| -> Result<_, ContractError> {
            match rates {
                Some(s) => {
                    let mut days: Vec<_> = s.into_iter().collect();
                    days.sort_by(compare_asc);
                    if days.len() > MAX_LEN_DAY {
                        days.truncate(MAX_LEN_DAY + 1);
                    }
                    let days_map: HashMap<String, RateCount> = days.into_iter().collect();
                    Ok(days_map)
                }
                None => {
                    let mut n = HashMap::new();
                    n.insert(
                        class_id.clone(),
                        RateCount {
                            ts,
                            sum: v as u64,
                            total: 1,
                        },
                    );
                    Ok(n)
                }
            }
        },
    )?;

    // update user stats
    USER_STATS.update(
        deps.storage,
        info.sender,
        |stats| -> Result<_, ContractError> {
            match stats {
                Some(s) => {
                    let mut st = s;
                    st.last_rate_ts = ts;
                    st.ratings = st.ratings.saturating_add(1);
                    Ok(st)
                }
                None => Ok(UserStats {
                    last_rate_ts: ts,
                    ratings: 1,
                    saves: 0,
                    shares: 0,
                }),
            }
        },
    )?;

    // update stats
    STATS.update(deps.storage, |mut stats| -> Result<_, ContractError> {
        stats.ratings = stats.ratings.saturating_add(1);
        Ok(stats)
    })?;

    Ok(Response::new().add_attribute("method", "rate"))
}

pub fn share(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    class_id: String,
    token: TokenUri,
    chain_id: Option<String>,
) -> Result<Response, ContractError> {
    // check NFT doesnt exist
    if NFTS.has(deps.storage, class_id.clone()) {
        return Err(ContractError::CustomError {
            val: "NFT already exists, cannot share again".to_string(),
        });
    }
    let c = CONFIG.load(deps.storage)?;

    // owner can share directly, otherwise check
    if info.sender != c.owner {
        let user_stats = USER_STATS.may_load(deps.storage, info.sender.clone())?;
        if let Some(user_stats) = user_stats {
            if c.unlock_share.unwrap_or(5) > user_stats.ratings {
                return Err(ContractError::CustomError {
                    val: "Not enough ratings, cannot share".to_string(),
                });
            }

            // Check max shares!!
            if user_stats.shares > DEFAULT_USER_MAX_SHARES {
                return Err(ContractError::CustomError {
                    val: "Maximum allowed shares reached, cannot share".to_string(),
                });
            }
        } else {
            return Err(ContractError::CustomError {
                val: "No user prefs, cannot share".to_string(),
            });
        }

        // check user provided adequate fee
        if !has_coins(&info.funds, &c.share_fee) {
            return Err(ContractError::CustomError {
                val: "Invalid share fee amount, cannot share".to_string(),
            });
        }
    }

    // create new NFT records
    LIST.push_back(deps.storage, &class_id)?;

    let nft = Nft {
        class_id: class_id.clone(),
        token,
        chain_id,
        index: Some(u64::from(LIST.len(deps.storage)?)),
    };

    NFTS.save(deps.storage, class_id, &nft)?;

    // update user stats
    USER_STATS.update(
        deps.storage,
        info.sender,
        |stats| -> Result<_, ContractError> {
            match stats {
                Some(s) => {
                    let mut st = s;
                    st.shares = st.shares.saturating_add(1);
                    Ok(st)
                }
                None => Ok(UserStats {
                    last_rate_ts: env.block.time.seconds(),
                    ratings: 0,
                    saves: 0,
                    shares: 1,
                }),
            }
        },
    )?;

    // update stats
    STATS.update(deps.storage, |mut stats| -> Result<_, ContractError> {
        stats.nfts = stats.nfts.saturating_add(1);
        Ok(stats)
    })?;

    Ok(Response::new().add_attribute("method", "share"))
}

pub fn save(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    class_id: String,
) -> Result<Response, ContractError> {
    // check NFT doesnt exist
    if !NFTS.has(deps.storage, class_id.clone()) {
        return Err(ContractError::CustomError {
            val: "NFT doesnt exist, cannot save".to_string(),
        });
    }
    let c = CONFIG.load(deps.storage)?;
    // check user provided adequate fee
    if info.sender != c.owner && !has_coins(&info.funds, &c.save_fee) {
        return Err(ContractError::CustomError {
            val: "Invalid save fee amount, cannot save".to_string(),
        });
    }

    let saved_list = USER_SAVED.may_load(deps.storage, info.sender.clone())?;

    match saved_list {
        Some(saved) => {
            // remove any matching class_ids
            let mut sl = saved;
            if !sl.contains(&class_id) {
                sl.push(class_id);
                USER_SAVED.save(deps.storage, info.sender.clone(), &sl)?;
            }
        }
        None => {
            USER_SAVED.save(deps.storage, info.sender.clone(), &vec![class_id])?;
        }
    }

    // update user stats
    USER_STATS.update(
        deps.storage,
        info.sender,
        |stats| -> Result<_, ContractError> {
            match stats {
                Some(s) => {
                    let mut st = s;
                    st.saves = st.saves.saturating_add(1);
                    Ok(st)
                }
                None => Ok(UserStats {
                    last_rate_ts: env.block.time.seconds(),
                    ratings: 0,
                    saves: 1,
                    shares: 0,
                }),
            }
        },
    )?;

    // update total stats
    STATS.update(deps.storage, |mut stats| -> Result<_, ContractError> {
        stats.saves = stats.saves.saturating_add(1);
        Ok(stats)
    })?;

    Ok(Response::new().add_attribute("method", "save"))
}

pub fn unsave(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    class_id: String,
) -> Result<Response, ContractError> {
    // check NFT exists
    if !NFTS.has(deps.storage, class_id.clone()) {
        return Err(ContractError::CustomError {
            val: "NFT doesnt exist, cannot unsave".to_string(),
        });
    }
    let saved_list = USER_SAVED.may_load(deps.storage, info.sender.clone())?;

    if let Some(saved_list) = saved_list {
        // remove any matching class_ids
        let mut sl = saved_list;
        sl.retain(|x| x.clone() != class_id);
        USER_SAVED.save(deps.storage, info.sender.clone(), &sl)?;

        // update user stats
        USER_STATS.update(
            deps.storage,
            info.sender,
            |stats| -> Result<_, ContractError> {
                match stats {
                    Some(s) => {
                        let mut st = s;
                        if st.saves > 0 {
                            st.saves = st.saves.saturating_sub(1);
                        }
                        Ok(st)
                    }
                    None => Ok(UserStats {
                        last_rate_ts: env.block.time.seconds(),
                        ratings: 0,
                        saves: 0,
                        shares: 0,
                    }),
                }
            },
        )?;

        // update total stats
        STATS.update(deps.storage, |mut stats| -> Result<_, ContractError> {
            stats.saves = stats.saves.saturating_sub(1);
            Ok(stats)
        })?;
    }

    Ok(Response::new().add_attribute("method", "unsave"))
}

pub fn send_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    class_id: String,
    message: String,
    meta: Option<Binary>,
) -> Result<Response, ContractError> {
    // check NFT exists
    if !NFTS.has(deps.storage, class_id.clone()) {
        return Err(ContractError::CustomError {
            val: "NFT doesnt exist, cannot message".to_string(),
        });
    }
    // check message length
    if message.len() > MAX_LEN_MESSAGE {
        return Err(ContractError::CustomError {
            val: "Message too long".to_string(),
        });
    }
    let c = CONFIG.load(deps.storage)?;
    // Get the prefs of receiver, to filter out thangs
    let user_stats = USER_STATS.may_load(deps.storage, info.sender.clone())?;
    if let Some(user_stats) = user_stats {
        if c.unlock_messages.unwrap_or(5) > user_stats.ratings {
            return Err(ContractError::CustomError {
                val: "Not enough ratings, cannot post".to_string(),
            });
        }
    } else {
        return Err(ContractError::CustomError {
            val: "No user prefs, cannot post".to_string(),
        });
    }

    let msg_id = env.block.time.seconds();
    let new_msg = Message {
        ts: msg_id,
        class_id: class_id.clone(),
        message,
        from: info.sender,
        meta,
    };

    // Get last index, push into new
    let stored_ids = MESSAGES_IDS.may_load(deps.storage, class_id.clone())?;
    let mut prev_ids = stored_ids.unwrap_or(vec![]);
    prev_ids.push(msg_id);
    MESSAGES.save(deps.storage, msg_id, &new_msg)?;
    MESSAGES_IDS.save(deps.storage, class_id, &prev_ids)?;

    // update stats
    STATS.update(deps.storage, |mut stats| -> Result<_, ContractError> {
        stats.messages = stats.messages.saturating_add(1);
        Ok(stats)
    })?;

    Ok(Response::new().add_attribute("method", "message"))
}

pub fn remove_message(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let c: Config = CONFIG.load(deps.storage)?;
    // Only owner can do withdraw
    if info.sender != c.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Get by index
    let msg = MESSAGES.may_load(deps.storage, id)?;
    MESSAGES.remove(deps.storage, id);
    if let Some(msg) = msg {
        let msgs = MESSAGES_IDS.may_load(deps.storage, msg.class_id.clone())?;
        if let Some(msgs) = msgs {
            // remove any matching msg ids
            let mut ms = msgs;
            ms.retain(|&x| x != id);
            MESSAGES_IDS.save(deps.storage, msg.class_id, &ms)?;
        }
    }

    // update stats
    STATS.update(deps.storage, |mut stats| -> Result<_, ContractError> {
        stats.messages = stats.messages.saturating_sub(1);
        Ok(stats)
    })?;

    Ok(Response::new().add_attribute("method", "remove_message"))
}

// Withdraw ALL funds to specified recipient (like a DAO)
pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let c: Config = CONFIG.load(deps.storage)?;
    // Only owner can do withdraw
    if info.sender != c.owner {
        return Err(ContractError::Unauthorized {});
    }
    let balances = deps.querier.query_all_balances(env.contract.address)?;
    let transfer = SubMsg::new(BankMsg::Send {
        to_address: deps.api.addr_validate(receiver.as_str())?.to_string(),
        amount: balances,
    });

    Ok(Response::new()
        .add_attribute("method", "withdraw")
        .add_submessage(transfer))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetList { kind, sort } => to_binary(&query_ranked_list(deps, env, kind, sort)?),
        QueryMsg::GetCurrentNft {} => to_binary(&query_current_nft(deps)?),
        QueryMsg::GetNftByIndex { index } => to_binary(&query_nft_by_index(deps, index)?),
        QueryMsg::GetNftByClassId { class_id } => {
            to_binary(&query_nft_by_class_id(deps, class_id)?)
        }
        QueryMsg::GetUserNftSaved { addr } => to_binary(&query_user_saved_nfts(deps, addr)?),
        QueryMsg::GetNftRate { class_id } => to_binary(&query_nft_rate(deps, class_id)?),
        QueryMsg::GetAllMessages { from_index, limit } => {
            to_binary(&query_all_messages(deps, from_index, limit)?)
        }
        QueryMsg::GetNftMessages { class_id } => to_binary(&query_messages(deps, class_id)?),
        QueryMsg::GetUser { addr } => to_binary(&query_user(deps, addr)?),
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetTotalStats {} => to_binary(&query_stats(deps)?),
        QueryMsg::GetClassId {
            contract_addr,
            token_id,
        } => to_binary(&query_class_id(contract_addr, token_id)?),
        QueryMsg::UserHasSavedNft { addr, class_id } => {
            to_binary(&query_user_saved_nft(deps, addr, class_id)?)
        }
        QueryMsg::GetUserNftRate { addr, class_id } => {
            to_binary(&query_user_rated_nft(deps, addr, class_id)?)
        }
    }
}

// NOTE: Response is unordered
fn query_ranked_list(
    deps: Deps,
    env: Env,
    kind: ListKind,
    sort: ListSort,
) -> StdResult<Vec<(Nft, RateCount)>> {
    let mut list: Vec<(Nft, RateCount)> = vec![];

    let keys: Vec<(String, RateCount)> = match kind {
        ListKind::All => match sort {
            ListSort::Highest => NFT_RATE_ATH
                .range(deps.storage, None, None, Order::Descending)
                .map(|res| res.unwrap())
                .collect::<Vec<(String, RateCount)>>(),
            ListSort::Lowest => NFT_RATE_ATL
                .range(deps.storage, None, None, Order::Descending)
                .map(|res| res.unwrap())
                .collect::<Vec<(String, RateCount)>>(),
        },
        ListKind::Day => {
            let ts = env.block.time.seconds();
            let day_remainder = ts % DAY_IN_SECONDS;
            let last_day_ts = ts.saturating_sub(day_remainder);

            match sort {
                ListSort::Highest => {
                    let values = NFT_RATE_DAY_ATH.load(deps.storage, last_day_ts)?;
                    values.into_iter().collect::<Vec<(String, RateCount)>>()
                }
                ListSort::Lowest => {
                    let values = NFT_RATE_DAY_ATL.load(deps.storage, last_day_ts)?;
                    values.into_iter().collect::<Vec<(String, RateCount)>>()
                }
            }
        }
        ListKind::Month => {
            let ts = env.block.time.seconds();
            let day_remainder = ts % DAY_IN_SECONDS;
            let last_day_ts = ts.saturating_sub(day_remainder);
            let last_month_ts = last_day_ts.saturating_sub(DAY_IN_SECONDS * 30);

            match sort {
                ListSort::Highest => NFT_RATE_DAY_ATH
                    .range(
                        deps.storage,
                        Some(Bound::inclusive(last_month_ts)),
                        Some(Bound::inclusive(last_day_ts)),
                        Order::Descending,
                    )
                    .filter_map(Result::ok)
                    .flat_map(|(_, rcs)| rcs.into_iter().collect::<Vec<(String, RateCount)>>())
                    .collect(),
                ListSort::Lowest => NFT_RATE_DAY_ATL
                    .range(
                        deps.storage,
                        Some(Bound::inclusive(last_month_ts)),
                        Some(Bound::inclusive(last_day_ts)),
                        Order::Descending,
                    )
                    .filter_map(Result::ok)
                    .flat_map(|(_, rcs)| rcs.into_iter().collect::<Vec<(String, RateCount)>>())
                    .collect(),
            }
        }
    };

    for (key, rc) in keys.into_iter() {
        let nft = NFTS.load(deps.storage, key)?;
        list.push((nft, rc));
    }

    Ok(list)
}

fn query_current_nft(deps: Deps) -> StdResult<Option<Nft>> {
    let i = LIST.back(deps.storage)?;
    if let Some(class_id) = i {
        let r = NFTS.may_load(deps.storage, class_id)?;
        Ok(r)
    } else {
        Ok(None)
    }
}

fn query_nft_by_index(deps: Deps, index: u32) -> StdResult<Option<Nft>> {
    let i = LIST.get(deps.storage, index)?;
    if let Some(class_id) = i {
        let r = NFTS.may_load(deps.storage, class_id)?;
        Ok(r)
    } else {
        Ok(None)
    }
}

fn query_nft_by_class_id(deps: Deps, class_id: String) -> StdResult<Option<Nft>> {
    let r = NFTS.may_load(deps.storage, class_id)?;
    Ok(r)
}

fn query_user_saved_nfts(deps: Deps, addr: Addr) -> StdResult<Vec<Nft>> {
    let saved = USER_SAVED.may_load(deps.storage, addr)?;
    if let Some(saved) = saved {
        let mut saved_nfts: Vec<Nft> = vec![];

        for class_id in saved.into_iter() {
            let nft = NFTS.may_load(deps.storage, class_id)?;
            if let Some(nft) = nft {
                saved_nfts.push(nft);
            }
        }

        Ok(saved_nfts)
    } else {
        Ok(vec![])
    }
}

fn query_user_saved_nft(deps: Deps, addr: Addr, class_id: String) -> StdResult<bool> {
    let saved = USER_SAVED.may_load(deps.storage, addr)?;
    if let Some(saved) = saved {
        Ok(saved.contains(&class_id))
    } else {
        Ok(false)
    }
}

fn query_user_rated_nft(deps: Deps, addr: Addr, class_id: String) -> StdResult<Option<Rate>> {
    NFT_RATINGS.may_load(deps.storage, (class_id, addr))
}

fn query_nft_rate(deps: Deps, class_id: String) -> StdResult<Option<RateCounts>> {
    let r = NFT_RATE_COUNTS.may_load(deps.storage, class_id)?;
    Ok(r)
}

fn query_all_messages(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<Message>> {
    let from_index = from_index.unwrap_or(0);
    let limit = limit.unwrap_or(100);

    Ok(MESSAGES
        .range(deps.storage, None, None, Order::Descending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|res| res.unwrap().1)
        .collect::<Vec<_>>())
}

fn query_messages(deps: Deps, class_id: String) -> StdResult<Vec<Message>> {
    let mids = MESSAGES_IDS.may_load(deps.storage, class_id)?;

    if let Some(msg_ids) = mids {
        let mut msgs: Vec<Message> = vec![];

        for ts in msg_ids.into_iter() {
            let msg = MESSAGES.may_load(deps.storage, ts)?;
            if let Some(msg) = msg {
                msgs.push(msg);
            }
        }

        Ok(msgs)
    } else {
        Ok(vec![])
    }
}

fn query_user(deps: Deps, addr: Addr) -> StdResult<Option<UserStats>> {
    let s = USER_STATS.may_load(deps.storage, addr)?;
    Ok(s)
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let c = CONFIG.load(deps.storage)?;
    Ok(c)
}

fn query_stats(deps: Deps) -> StdResult<TotalStats> {
    let s = STATS.load(deps.storage)?;
    Ok(s)
}

fn query_class_id(contract_addr: String, token_id: String) -> StdResult<String> {
    Ok(format!("{}{}", contract_addr, token_id))
}
