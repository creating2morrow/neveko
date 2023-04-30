# The Manual

## Architecture

* gui
* three internal mircoservers (auth, contact and message)
* core code module and lmdb
* one external i2p hidden service
* jwt for internal auth, jwp for external

### JWP (JSON Web Proof)

* utilizes some external blockchain (nevmes uses monero) for authorization of auth tokens
* 32 byte random signing keys generated on app start-up
* `Hmac<Sha384>` internal, `Hmac<Sha512>` external (jwp)
* see [proof.rs](./nevmes-core/src/proof.rs)
`eyJhbGciOiJIUzUxMiJ9.eyJhZGRyZXNzIjoiNThvaUJMQUtBQ3JaeTRqVnRYdUFXMzlCOW1zR3dlbVVkSm9HVlozcGdSY1RoWHZqWjZ0RERqRGpuOE1mTUZ5cEtZMlU1U1B6SkE3NnFHeHhDdjJzd1Y0NjhFYkI2dEsiLCJoYXNoIjoiNzRhOTM5NTU1Y2EyMWJmY2MxYzlhMjhlYjFkN2M5MWZiMjRhYzRiOTY4MDk2Yzg4ODU1ODA3ODcwMDA1NmQ2NiIsIm1lc3NhZ2UiOiIiLCJzaWduYXR1cmUiOiJPdXRQcm9vZlYyWHdYTEJYV0VtbXlWd3YyOHFQRWQ0Mk14bm1FNTU3aUFEVHFGNjZDWG9LQ1ZFeFBqTVU4NFNIeWprZmdLd01WZEI4OUZkTkJ5QUxyeU1ZamVxQlY1U0VtU0V4MUJWWE1ITVJNWHVuMzh5aWVtcWhCcmVSWUdpRGdMN1lmRmVmemJSTnhlIn0.gH4RlLrxu3xqxNvsHv7lX1yYomg07yTlv6VEKpDfXwbDV4O267CXzm30G4YBQOfuDf3xpegUmeVXOScPvIZVRw`

## Getting started

* getting started the app will automatically generate an account and associated monero PRIMARY address. Only use it here to maintain privacy
* so first off, you need to someone to love, dont you want somebody to love...
* get your contact's i2p .b32.i2p address (top of gui screen)
* deposit some stagenet monero in your xmr account (address at top of gui screen)
* once unlocked nevmes xmr balance will display
* when authorizing to send to contact an invoice will be generated
* authorize payment and tx proof generation in the prompt
* this tx proof will be used to create an encrypted json web proof of payment with each contact
* think of it as a reusable, unforgeable coupon or ticket
* the invoice shows payment per blocks (time)
* default is 1 piconero per day
* the jwp is cached by the client until block time expiration at which time you will be required to authorize another payment

## Adding a contact
* now that the bills are paid insert the .b32.i2p address on the contact line
* click add
* if all goes well you will have imported their public nevmes gpg app key
* dont reuse the app gpg keys anywhere else!
* don't forget to trust the contact with `sign key` in the `check status` window

## Sending a message

* the `check status` button will show current jwp for each contact
* `clear stale jwp` will purge data in case of timeout issues
* don't keep large amounts in nevmes just enoug for fees and jwps
* once a valid jwp is created (takes a few minutes) the `compose` button will be visible
* you need to click `check status` on contacts before sending to refresh jwp expiration check
* draft a plain text message, dont be shy
* enter recipient (.b32.i2p address) and press `send`
* plain text messages never leave your machine
* you can click `Refresh` button in the Mailbox to check for new messages
* messages must be decrypted by clicking `decrypt`

## More to come...
