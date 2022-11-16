## EXCHANGE_PALLET
---
# Calls
- fn submit_order
    + base_currency_id: CurrencyId
    + base_amount: Balance
    + target_currency_id: CurrencyId
    + target_amount: Balance
- fn take_order
    + order_id: OrderId
- fn cancel_order
    + order_id: OrderId

# Types
- struct Order
    + base_currency_id: CurrencyId
    + base_amount: Balance
    + target_currency_id: CurrencyId
    + target_amount: Balance
    + owner: AccountId

# Storage
- Orders: map OrderId => Order
- NextOrderId: OrderId

# Events
- OrderCreated(OrderId, Order)
- OrderTaken(AccountId, OrderId, Order)
- OrderCancelled(OrderId)