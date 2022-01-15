# zero2prod

Following the zero2prod book using rocket.

TODO:
- Chapter 7 exercises.
- Chapter 10:  
  - Add validation for password: less than 12 chars or longer than 128.
  - Add a Send a newsletter issue link to the admin dashboard;
  - Add an HTML form at GET /admin/newsletters to submit a new issue;
  - Adapt POST /newsletters to process the form data:
    - Change the route to POST /admin/newsletters;
    - Migrate from ‘Basic’ to session-based authentication;
    - Migrate the expected input from application/json to application/x-www-form-urlencoded;
    - Adapt the test suite.
