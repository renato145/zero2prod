# zero2prod

Following the zero2prod book using rocket.

TODO:
- Chapter 7:
  - What happens if a user tries to subscribe twice? Make sure that they receive two confirmation emails;
  - What happens if a user clicks on a confirmation link twice?
  - What happens if the subscription token is well-formatted but non-existent?
  - Add validation on the incoming token, we are currently passing the raw user input straight into a
    query (thanks sqlx for protecting us from SQL injections <3);
- Chapter 11:
  - GET /admin/newsletters: redirect the user back to the form page with a proper error message when body
    validation fails (page 444).
  - Delivery process (a page to track how many emails are still) outstanding for a certain newsletter
    issue (page 474).
  - Retry when the delivery attempt fails due to a Postmark error. Enhance issue_delivery_queue e.g. adding
    a `n_retries` and `execute_after` columns to keep track of how many attempts have already taken place
    and how long we should wait before trying again (page 480).
  - Almost all errors returned by try_execute_task are transient in nature, except for invalid subscriber
    emails - sleeping is not going to fix those. Try refining the implementation to distinguish between
    transient and fatal failures, empowering worker_loop to react appropriately (page 481).
  - Expiry mechanism for idempotency keys. Try designing one using background workers (page 487).

