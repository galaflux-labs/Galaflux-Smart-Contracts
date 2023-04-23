use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, IdsResponse, InstantiateMsg, QueryMsg, ReceiveMsg, StreamResponse};
use crate::state::{save_stream, Config, Stream, CONFIG, STREAMS, STREAM_SEQ, USERS_STREAMS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;
use cw20::{Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};

const CONTRACT_NAME: &str = "crates.io:cw-stream";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
        .unwrap_or(info.sender);
    let config = Config {
        owner: owner.clone(),
        cw20_addr: deps.api.addr_validate(msg.cw20_addr.as_str())?,
    };
    CONFIG.save(deps.storage, &config)?;

    let start_ind: u64 = 0;
    STREAM_SEQ.save(deps.storage, &start_ind)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("cw20_addr", msg.cw20_addr))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => execute_receive(env, deps, info, msg),
        ExecuteMsg::Withdraw { id } => try_withdraw(env, deps, info, id),
    }
}

pub fn try_create_stream(
    env: Env,
    deps: DepsMut,
    owner: String,
    recipient: String,
    amount: Uint128,
    start_time: u64,
    end_time: u64,
) -> Result<Response, ContractError> {
    let validated_owner = deps.api.addr_validate(owner.as_str())?;
    if validated_owner != owner {
        return Err(ContractError::InvalidOwner {});
    }

    let validated_recipient = deps.api.addr_validate(recipient.as_str())?;
    if validated_recipient != recipient {
        return Err(ContractError::InvalidRecipient {});
    }

    let config = CONFIG.load(deps.storage)?;
    if config.owner == recipient {
        return Err(ContractError::InvalidRecipient {});
    }

    let block_time = env.block.time.seconds();
    let start_time_real;
    if start_time == 0 {
        start_time_real = block_time;
    } else {
        start_time_real = start_time
    }

    if start_time_real > end_time {
        return Err(ContractError::InvalidStartTime {});
    }

    if start_time_real < block_time {
        return Err(ContractError::InvalidStartTime {});
    }

    let duration: Uint128 = end_time.checked_sub(start_time_real).unwrap().into();

    if amount < duration {
        return Err(ContractError::InvalidDuration {});
    }

    let last_amount = amount.u128().checked_rem(duration.u128()).unwrap();
    let real_amount = amount.u128().checked_sub(last_amount).unwrap();

    let rate_per_second: Uint128 = real_amount.checked_div(duration.u128()).unwrap().into();

    let stream = Stream {
        owner: validated_owner,
        recipient: validated_recipient,
        amount,
        last_amount: Uint128::from(last_amount),
        claimed_amount: Uint128::zero(),
        start_time: start_time_real,
        end_time,
        rate_per_second,
    };
    save_stream(deps, &stream)?;

    Ok(Response::new()
        .add_attribute("method", "try_create_stream")
        .add_attribute("owner", owner)
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount)
        .add_attribute("start_time", start_time.to_string())
        .add_attribute("end_time", end_time.to_string()))
}

pub fn execute_receive(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
    wrapped: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.cw20_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let msg: ReceiveMsg = from_binary(&wrapped.msg)?;
    match msg {
        ReceiveMsg::CreateStream {
            recipient,
            start_time,
            end_time,
        } => try_create_stream(
            env,
            deps,
            wrapped.sender,
            recipient,
            wrapped.amount,
            start_time,
            end_time,
        ),
    }
}

pub fn try_withdraw(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, id)?;
    if stream.recipient != info.sender {
        return Err(ContractError::NotStreamRecipient {});
    }

    if stream.claimed_amount >= stream.amount {
        return Err(ContractError::StreamFullyClaimed {});
    }

    let block_time = env.block.time.seconds();
    if stream.start_time >= block_time {
        return Err(ContractError::StreamNotStarted {});
    }

    let mut last_amount: Uint128 = Uint128::from(0 as u64);
    if stream.end_time <= block_time {
        last_amount = stream.last_amount;
    }


    let unclaimed_amount = u128::from(block_time)
        .checked_sub(stream.start_time.into())
        .unwrap()
        .checked_mul(stream.rate_per_second.u128())
        .unwrap()
        .checked_sub(stream.claimed_amount.u128())
        .unwrap()
        .checked_add(last_amount.u128())
        .unwrap();

    stream.claimed_amount = stream
        .claimed_amount
        .u128()
        .checked_add(unclaimed_amount)
        .unwrap()
        .into();

    STREAMS.save(deps.storage, id, &stream)?;

    let config = CONFIG.load(deps.storage)?;
    let cw20 = Cw20Contract(config.cw20_addr);
    let msg = cw20.call(Cw20ExecuteMsg::Transfer {
        recipient: stream.recipient.to_string(),
        amount: unclaimed_amount.into(),
    })?;

    let res = Response::new()
        .add_attribute("method", "try_withdraw")
        .add_attribute("stream_id", id.to_string())
        .add_attribute("amount", Uint128::from(unclaimed_amount))
        .add_attribute("recipient", stream.recipient.to_string())
        .add_message(msg);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetStream { id } => to_binary(&query_stream(deps, id)?),
        QueryMsg::GetIds { addr } => to_binary(&query_ids(deps, addr)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.into_string(),
        cw20_addr: config.cw20_addr.into_string(),
    })
}

fn query_stream(deps: Deps, id: u64) -> StdResult<StreamResponse> {
    let stream = STREAMS.load(deps.storage, id)?;
    Ok(StreamResponse {
        owner: stream.owner.into_string(),
        recipient: stream.recipient.into_string(),
        amount: stream.amount,
        claimed_amount: stream.claimed_amount,
        rate_per_second: stream.rate_per_second,
        start_time: stream.start_time,
        end_time: stream.end_time,
    })
}


fn query_ids(deps: Deps, addr: String) -> StdResult<IdsResponse> {
    let validated_owner = deps.api.addr_validate(addr.as_str())?;
    let ids = USERS_STREAMS.may_load(deps.storage, validated_owner)?;
    if ids.is_some() {
        Ok(IdsResponse {
            ids: ids.unwrap()
        })
    } else {
        Ok(IdsResponse {
            ids: Vec::new()
        })
    }
}
