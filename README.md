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
