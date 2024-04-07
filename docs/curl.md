# Remote access

NOTE: JWT for micro servers disabled in dev

## Login API

* send with dummy string on first request
* sign data in the response
* send second request with signature to get AUTHID and UID

```bash
curl -iv -x localhost:9043/login/<SIGNATURE>/<AUTHID>/<UID>
```

## generate invoice

```bash
curl -iv  http://bob.b32.i2p/invoice
```

## get contact info

```bash
curl -iv  http://bob.b32.i2p/share
```

## generate jwp

```bash
curl -iv -X POST http://bob.b32.i2p/prove -d '{"address": "", "confirmations":0,"hash":"", "message":"", "signature": ""}' -H 'Content-Type: application/json'
```

## health check

```bash
curl -iv http://bob.b32.i2p/xmr/version -H 'proof: eyJhbGciOiJIUzUxMiJ9...'
```

## add contact

```bash
curl -iv -X POST http://localhost:9044/contact -d '{"cid": "KEEP EMPTY", "npmk": "string", "i2p_address": "", "xmr_address": ""}' -H 'Content-Type: application/json' 
```

## view contacts

```bash
curl -iv http://localhost:9044/contacts
```

## send message

```bash
curl -ivk http://localhost:9045/tx -d '{"uid":"123", "mid": "", "body": [1,2,3 <PLAINTEXT_BYTES>], "from": "alice.b32.i2p", "created": 0, "to": "bob.b32.i2p"}' -H 'Content-Type: application/json'
```

## receive message

```bash
curl -iv http://alice.b32.i2p/message/rx -d '{"uid":"", "mid": "", "body": [1,2,3 <ENCRYPTED_BYTES>], "from": "bob.b32.i2p", "created": 0, "to": "alice.b32.i2p"}' -H 'Content-Type: application/json' -H 'proof: eyJhbGciOiJIUzUxMiJ9...'
```

## view messages

```bash
curl -iv http://localhost:9045/messages
```

## decipher message

```bash
curl -iv http://localhost:9045/message/decipher/<MESSAGE_ID>
```