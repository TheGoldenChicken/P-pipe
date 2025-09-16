# Database considerations

- Find out if PostgreSQL makes sense...
  - Not large-scale
  - Maybe don't even use an SQL-like database?
  - You know how to use it, tho
- Find out if we need more like a data-lake format?
  - Keeps .json files, makes sense if what we want to keep is a list of "transactions"


Data to actually keep in the database
- Original data uploaded?
  - Easiest if we keep it ourselves
  - Will allow for easy splitting and everything
  -  Will take up much space, though.
  -  Alternative is keeping a "link" to the data, so requesting the original owner every time we need access
     -  Infeasible, that owner may be inaccessible at those times
     -  Requires user to set up bespoke solutions, infeasible
-  Challenges "on"
   -  Active or previously active - are more data transactions, or evaluations pending?
   -  Started at
   -  (Expected) Ends at
   -  Full list of commands to be given to dispatcher 
-  "Transactions"
   -  When was given
   -  When was performed
   -  Status (success, failure, error?)
   -  Stdout, stderr result of the transaction
   -  User confirmation recieved (if any, so possibly, None)
   -  String or otherwise, indicating *where* data was placed (address to database, drive, S3 bucket, etc. )
-  

Find out how to structure the .jsons that control single transactions

Also need some way of keeping track of other side of "transactions" - those that expect something from user?