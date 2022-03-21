# zero2prod

Following the zero2prod book using rocket.

TODO:
- Chapter 11:
  - Retry when the delivery attempt fails due to a Postmark error. Enhance issue_delivery_queue e.g. adding
    a `n_retries` and `execute_after` columns to keep track of how many attempts have already taken place
    and how long we should wait before trying again (page 480).
  - Almost all errors returned by try_execute_task are transient in nature, except for invalid subscriber
    emails - sleeping is not going to fix those. Try refining the implementation to distinguish between
    transient and fatal failures, empowering worker_loop to react appropriately (page 481).
  - Expiry mechanism for idempotency keys. Try designing one using background workers (page 487).

