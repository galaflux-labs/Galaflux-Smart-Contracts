## Stream payments protocol

### Build and deploy smart contract
Deploy CW-20 token contract before, and then deploy stream payment smart contract:
```bash
archway build
archway deploy
archway instantiate --args '{"cw20_addr": "archway1er6zgxw8dusgqvx6jzkcpfkscnlm5vfg97hj43hl4cy4px7l5xss575vey"}'
```

### Interact with CW-20 token smart contract

#### Create stream
contract - address of the stream protocol contract

amount - token amount

msg - encoded info
```js
console.log(Buffer.from(JSON.stringify({ "create_stream": { "recipient": "archway1g0eqw6amzwxdprsarxl5q2t6zgcne354q37q0w", "start_time": 0, "end_time": 1683272678}})).toString('base64'))
>> eyJjcmVhdGVfc3RyZWFtIjp7InJlY2lwaWVudCI6ImFyY2h3YXkxZzBlcXc2YW16d3hkcHJzYXJ4bDVxMnQ2emdjbmUzNTRxMzdxMHciLCJzdGFydF90aW1lIjowLCJlbmRfdGltZSI6MTY4MzI3MjY3OH19
```

```bash
archway tx --args '{ "send": {"contract": "archway1c3esfmxrwwly792y262dcggfc20fjm5g22ql7agpwnaf9ga4td4qz4c6rq", "amount": "1000000000000000000", "msg": "eyJjcmVhdGVfc3RyZWFtIjp7InJlY2lwaWVudCI6ImFyY2h3YXkxZzBlcXc2YW16d3hkcHJzYXJ4bDVxMnQ2emdjbmUzNTRxMzdxMHciLCJzdGFydF90aW1lIjowLCJlbmRfdGltZSI6MTY4MzI3MjY3OH19"}}' --flags --gas 300000
```


### Interact with stream payment smart contract

#### Get stream info
```bash
archway query contract-state smart --contract archway1m0xf0xhjg6flv9ycsqcruq6w639uuca7rwksq2c0x3ss4vpzu76qfzz2al --args '{ "get_stream": {"id": 1} }'
archway query contract-state smart --contract archway1m0xf0xhjg6flv9ycsqcruq6w639uuca7rwksq2c0x3ss4vpzu76qfzz2al --args '{ "get_ids": {"addr": "archway1g0eqw6amzwxdprsarxl5q2t6zgcne354q37q0w"} }'
```

#### Withdraw the part of the stream that is already unlocked
```bash
archway tx --args '{ "withdraw": {"id": 1}}' --flags --gas 300000
```
