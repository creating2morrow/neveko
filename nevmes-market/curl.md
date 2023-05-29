# CURL / API docs

```bash
# [GET] get the current monero-wallet-rpc version
curl http://127.0.0.1:8000/xmr/version

# [POST] sign data (monero-wallet-rpc)
curl http://127.0.0.1:38083/json_rpc --digest -u user:pass -d '{"jsonrpc":"2.0","id":"0","method":"sign","params":{"data":"some data here"}}' -H 'Content-Type: application/json'

# [POST] get addresses (monero-wallet-rpc)
curl http://127.0.0.1:38083/json_rpc --digest -u user:pass -d '{"jsonrpc":"2.0","id":"0","method":"get_address","params":{"account_index":0,"address_index":[0]}}' -H 'Content-Type: application/json'

# [GET] login
# customer or vendor
# xmr address
# aid - auth id
# cvid - customer or vendor id (2nd api call finalizes login and creates it)
# data - random bytes to sign
# signature - generate signature with wallet private keys
curl http://127.0.0.1:8000/login/<customer|vendor>/<XMR_ADDRESS>/<SIGNATURE>/<AID>/CID

# [GET] information
# customer or vendor
# xmr address
# customer or vendor id
curl http://127.0.0.1:8000/<customer|vendor>/<XMR_ADDRESS>/<ID> -H 'token: <JWT>'

# [PATCH] update
# customer or vendor URI
# <id> - i32
# <data> - String
# <update_type> - Enum => 0 - active, 1 - description, 2 - name, 3 - pgp
curl -iv -X PATCH http://127.0.0.1:8000/<customer|vendor>/<XMR_ADDRESS>/update -d '{"cid": "CID", "name": "<name>", "pgp": "<pgp>", "xmr_address": "" }'

# [GET]
# create a new product
curl -iv http://127.0.0.1:8000/product/<XMR_ADDRESS>/create -H 'token: <JWT>'

# [GET]
# return all products for a vendor
curl -iv http://127.0.0.1:8000/products/<XMR_ADDRESS> -H 'token: <JWT>'

# [PATCH] update product
# <pid> - i32
# <data> - String
# <update_type> - Enum => 0 - in_stock, 1 - description, 2 - name, 3 - price 4 - qty
curl -X PATCH http://127.0.0.1:8000/product/<XMR_ADDRESS>/update/<pid>/<data>/<update_type> -H 'token: <JWT>'

# [GET]
# intialize an order for a customer
curl -iv http://127.0.0.1:8000/order/<XMR_ADDRESS>/create/<pid> -H 'token: <JWT>'

# [GET]
# get all orders
# xmr address
# customer | vendor
curl -iv http://127.0.0.1:8000/orders/<XMR_ADDRESS>/<customer | vendor> -H 'token: <JWT>'

# [PATCH]
# modify order
#           UpdateType::CustomerKex1 => 0,         // make output from customer
#           UpdateType::CustomerKex2 => 1,         // use this for funding kex
#           UpdateType::CustomerKex3 => 2,         // might need this later?
#           UpdateType::CustomerMultisigInfo => 3, // prepare output from customer
#           UpdateType::Deliver => 4,              // customer has received the item, released txset
#           UpdateType::Hash => 5,                 // tx hash from funding the wallet order
#           UpdateType::Ship => 6,                 // update ship date, app doesn't store tracking numbers
#           UpdateType::Subaddress => 7,           // update address for payout
#           UpdateType::VendorKex1 => 8,           // make output from vendor
#           UpdateType::VendorKex2 => 9,           // use this for funding kex
#           UpdateType::VendorKex3 => 10,          // might need this later?
#           UpdateType::VendorMultisigInfo => 11,  // prepare output from vendor
#           UpdateType::Quantity => 12,            // this can be updated until wallet is funded
curl -X PATCH http://127.0.0.1:8000/order/<XMR_ADDRESS>/update/<pid>/<oid>/<data>/<update_type> -H 'token: <JWT>'
```
