use std::u64;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub cw20_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stream {
    pub owner: Addr,
    pub recipient: Addr,
    pub amount: Uint128,
    pub last_amount: Uint128,
    pub claimed_amount: Uint128,
    pub start_time: u64,
    pub end_time: u64,
    pub rate_per_second: Uint128,
}

pub const STREAM_SEQ: Item<u64> = Item::new("stream_seq");
pub const STREAMS: Map<u64, Stream> = Map::new("stream");

pub const USERS_STREAMS: Map<Addr, Vec<u64>> = Map::new("users_stream");

pub fn save_stream(deps: DepsMut, stream: &Stream) -> StdResult<()> {
    let mut id = STREAM_SEQ.load(deps.storage)?;
    id = id + 1;

    let res = USERS_STREAMS.may_load(deps.storage, stream.clone().owner)?;
    let mut ids: Vec<u64>;
    if res.is_some(){
        ids = res.unwrap()
    } else {
        ids = Vec::new();
    }
    ids.push(id);
    USERS_STREAMS.save(deps.storage, stream.clone().owner, &ids)?;

    let r_res = USERS_STREAMS.may_load(deps.storage, stream.clone().recipient)?;
    let mut r_ids: Vec<u64>;
    if r_res.is_some(){
        r_ids = r_res.unwrap()
    } else {
        r_ids = Vec::new();
    }
    r_ids.push(id);
    USERS_STREAMS.save(deps.storage, stream.clone().recipient, &r_ids)?;

    STREAM_SEQ.save(deps.storage, &id)?;
    STREAMS.save(deps.storage, id, stream)
}

