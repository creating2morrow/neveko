# Remote access

NOTE: JWT for micro servers disabled in dev

## Login API

* send with dummy string on first request
* sign data in the response
* send second request with signature to get AUTHID and UID

```bash
curl -iv -x localhost:9043/<XMR_ADDRESS>/login/<SIGNATURE>/<AUTHID>/<UID>
```

## generate invoice

```bash
curl -iv  http://localhost:9000/invoice
```

## get contact info

```bash
curl -iv  http://localhost:9000/share
```

## generate jwp

```bash
curl -iv -X POST http://localhost:9000/prove -d '{"address": "", "confirmations":0,"hash":"", "message":"", "signature": ""}' -H 'Content-Type: application/json'
```

## health check

```bash
curl -iv http://localhost:9000/xmr/version -H 'proof: eyJhbGciOiJIUzUxMiJ9...'
```

## add contact

```bash
curl -iv -X POST http://localhost:9044/contact -d '{"cid": "KEEP EMPTY", "gpg_key": [1,2,3...], "i2p_address": "", "xmr_address": ""}' -H 'Content-Type: application/json'
```

## view contacts

```bash
curl -iv http://localhost:9044/contacts
```

## send message

```bash
curl -ivk localhost:9045/tx -d '{"uid":"123", "mid": "", "body": [1,2,3 <PLAINTEXT_BYTES>], "from": "alice.b32.i2p", "created": 0, "to": "bob.b32.i2p"}' -H 'Content-Type: application/json'
```

## receive message

```bash
curl -ivk localhost:9000/message/rx -d '{"uid":"", "mid": "", "body": [1,2,3 <ENCRYPTED_BYTES>], "from": "alice.b32.i2p", "created": 0, "to": "bob.b32.i2p"}' -H 'Content-Type: application/json' -H 'proof: eyJhbGciOiJIUzUxMiJ9...'
```

## view messages

```bash
curl -iv http://localhost:9044/messages
```
