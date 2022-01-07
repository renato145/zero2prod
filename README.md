# zero2prod

Following the zero2prod book using rocket.

TODO:
- Chapter 7 exercises.
- Chapter 10:  
  - An alternative approach, to spare us the repetition, is to create a middleware that wraps all the endpoints nested
    under the /admin/ preﬁx. The middleware checks the session state and redirects the visitor to /login if they are not
    logged in. If you like a challenge, give it a try! Beware though: actix-web’s middlewares can be tricky to implement
    due to the lack of async syntax in traits.
  - Add validation for password: less than 12 chars or longer than 128.
  - Add a Send a newsletter issue link to the admin dashboard;
  - Add an HTML form at GET /admin/newsletters to submit a new issue;
  - Adapt POST /newsletters to process the form data:
    - Change the route to POST /admin/newsletters;
    - Migrate from ‘Basic’ to session-based authentication;
    - Migrate the expected input from application/json to application/x-www-form-urlencoded;
    - Adapt the test suite.
