# neveko-market

![market](../assets/vendor.png)

## High-Level Order Lifecycle

* vendor adds a new product with description and price
* customer orders product
* vendor creates order, multisig wallet

|        | prepare | make | exchange |
|--      |--       |--    |--        |        
|vend    |         |      |          |          
|cust    |         |      |          |          
|med     |         |      |          |          

* customer creates multisig wallet and prepares while collecting participant info

|        | prepare | make | exchange |
|--      |--       |--    |--        |        
|vend    |     X   |      |          |          
|cust    |     X   |      |          |          
|med     |     x   |      |          |          

* customer makes and sends both prepare infos to mediator and vendor
* participants all make_info

|        | prepare | make | exchange |
|--      |--       |--    |--        |
|vend    |     X   |   x  |          |
|cust    |     X   |   x  |          |
|med     |     X   |   X  |          |

* customer calls to exchange multisig keys and collects outputs again

|        | prepare | make | exchange |
|--      |--       |--    |--        |
|vend    |     X   |   X  |          |
|cust    |     X   |   X  |      X   |
|med     |     X   |   X  |          |

* customer sends output to participants who then exchange multisig keys

|        | prepare | make | exchange |
|--      |--       |--    |--        |
|vend    |     X   |   X  |      X   |
|cust    |     X   |   X  |      X   |
|med     |     X   |   X  |      X   |
        
* customer funds wallet and exports info to vendor and mediator
* vendor and mediator import multisig info
* customer signs multisig txset and sends to mediator
* mediator requests tracking number from vendor
* mediator relase signed txset to vendor
* vendor signs and submits signed txset
* in case of dispute the mediator can sign multisig txset for customer refund

Reference: https://resilience365.com/monero-multisig-how-to/
