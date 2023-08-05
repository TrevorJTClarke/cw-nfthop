<div align="center">
  <p>Testing Flows & Messages</p>
</div>

Great way to test these is:

1. Upload & Instantiate at [cw-tools.vercel.app/](https://cw-tools.vercel.app/)
2. Execute then Query

# Core Flow

### 1. Instantiate

NOTE: Change the values to reflect deployed contracts

**EXEC**
```json
{
  "share_fee": {
    "amount": "1000000",
    "denom": "ustars"
  },
  "save_fee": {
    "amount": "1000000",
    "denom": "ustars"
  }
}
```

**QUERY**
```json
{
  "get_config": {}
}
```

### 2. Share NFT

**EXEC**
```json
{
  "share": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652",
    "token": {
      "contract_addr": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n420",
      "id": "2652",
      "data_uri": "ipfs://QmUoHk4hY6mNoHgNEJDcy94APUky6o8xVmyD3YzddJtUWe/metadata/2652"
    }
  }
}
```

NOTE: You can check the data_uri here: https://ipfs-gw.stargaze-apis.com/ipfs/QmUoHk4hY6mNoHgNEJDcy94APUky6o8xVmyD3YzddJtUWe/2652

**QUERY**
```json
{
  "get_nft_by_class_id": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

### 3. Rate NFT

**EXEC**
```json
{
  "rate": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652",
    "v": 5
  }
}
```

**QUERY**
```json
{
  "get_nft_rate": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

### 4. Add Message to NFT

**EXEC**
```json
{
  "message": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652",
    "message": "BURN IT ALL MUAHAHAHHAHAHHAHAAHHAHAHAHA! 🔥"
  }
}
```

**QUERY**
```json
{
  "get_nft_messages": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

### 5. User Save NFT

**EXEC**
```json
{
  "save": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

**QUERY**
```json
{
  "get_user_nft_saved": {
    "addr": "stars1234334"
  }
}
```

**EXEC**
```json
{
  "unsave": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

-----

# Extra Flows/Messages

### Get Ranked Lists

* **Kinds**: "all", "day", "month"

* **Sorts**: "highest", "lowest"

```json
{
  "get_list": {
    "kind": "all",
    "sort": "highest"
  }
}
```

### Get Current NFT

```json
{
  "get_current_nft": {}
}
```

### Get NFT by Index

```json
{
  "get_nft_by_index": {
    "index": 1
  }
}
```

### Get NFT by Class Id

```json
{
  "get_nft_by_class_id": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

### Get NFT Rate

```json
{
  "get_nft_rate": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

### Get All Messages

```json
{
  "get_all_messages": {
    "from_index": 0,
    "limit": 10
  }
}
```

### Get NFT Messages

```json
{
  "get_nft_messages": {
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

### Get User

```json
{
  "get_user": {
    "addr": "stars1234334"
  }
}
```

### Get User NFTs Saved

```json
{
  "get_user_nft_saved": {
    "addr": "stars1234334"
  }
}
```

### User Has Saved NFT

```json
{
  "user_has_saved_nft": {
    "addr": "stars1234334",
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

### Get User NFT Rate

```json
{
  "get_user_nft_rate": {
    "addr": "stars1234334",
    "class_id": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n4202652"
  }
}
```

### Get Total Stats

```json
{
  "get_total_stats": {}
}
```

### Get Config

```json
{
  "get_config": {}
}
```

### Get Class Id

```json
{
  "get_class_id": {
    "contract_addr": "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n420",
    "token_id": "2652"
  }
}
```

### Owner remove message

**EXEC**
```json
{
  "remove_message": {
    "id": 1
  }
}
```

### Owner Withdraw Balances

**EXEC**
```json
{
  "withdraw": {
    "receiver": "stars1234334"
  }
}
```