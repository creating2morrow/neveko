# nevmes-market

## High-Level Order Lifecycle

* vendor adds a new product with description and price
* customer orders product
* vendor creates order, multisig wallet and prepares it

|        | prepare | make | exchange |
|--      |--       |--    |--        |        
|vend    |     X   |      |          |          
|cust    |         |      |          |          
|med     |         |      |          |          

* customer saves prepare info from vendor, creates multisig wallet

|        | prepare | make | exchange |
|--      |--       |--    |--        |        
|vend    |     X   |      |          |          
|cust    |     X   |      |          |          
|med     |         |      |          |          

* customer sends both prepare infos to mediator
* mediator creates multisig wallet, prepares and makes it

|        | prepare | make | exchange |
|--      |--       |--    |--        |
|vend    |     X   |      |          |
|cust    |     X   |      |          |
|med     |     X   |   X  |          |

* customer makes multisig wallet and sends both outputs to vendor

|        | prepare | make | exchange |
|--      |--       |--    |--        |
|vend    |     X   |      |          |
|cust    |     X   |   X  |          |
|med     |     X   |   X  |          |

* vendor makes and calls to exchange multisig keys

|        | prepare | make | exchange |
|--      |--       |--    |--        |
|vend    |     X   |   X  |      X   |
|cust    |     X   |   X  |          |
|med     |     X   |   X  |          |

* customer sends output to mediator who then exchanges multisig keys

|        | prepare | make | exchange |
|--      |--       |--    |--        |
|vend    |     X   |   X  |      X   |
|cust    |     X   |   X  |      X   |
|med     |     X   |   X  |      X   |
        
* customer funds wallet and exports to vendor and mediator
* vendor and mediator import multisig info
* customer signs multisig txset and sends to mediator
* mediator requests tracking number from vendor
* mediator relase signed txset to vendor
* vendor signs and submits signed txset
* in case of dispute the mediator can sign multisig txset for customer refund

Reference: https://resilience365.com/monero-multisig-how-to/
