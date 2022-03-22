# zero2prod

Following the zero2prod book using rocket.

TODO:
- Chapter 11:
  - Almost all errors returned by try_execute_task are transient in nature, except for invalid subscriber
    emails - sleeping is not going to fix those. Try refining the implementation to distinguish between
    transient and fatal failures, empowering worker_loop to react appropriately (page 481).
  - Expiry mechanism for idempotency keys. Try designing one using background workers (page 487).

