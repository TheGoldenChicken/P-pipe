# P-PIPE Plan Going forward

## MVP required features

- Automatic access bindings assignment
- A frontend



## Extra, nice-to-have features


## DevOps features

Pre-commit

## Feature planning implementation

### AABA - "The Assigner"

Must:
- Create mail to send to users with access information for various things
- Create credentials for every third location
- Include descriptions, names, important information, PDFs, etc. that must be sent to users

Thought about it a bit:
- Should focus on making everything in Rust. It should be possible, and *if* we are going to remove Python at some point, it just makes sense to do that...
- 1. Rust creates the mail
- 2. For each dispatch location:
  - Rust calls a function - `get_access_stuff` or smth. That'll get the specific string that we're looking for with the access
- However, right now, we don't know if you can give access to folders and such that do not exist.
- This might mean we have to create the folders and everything as soon as we instantiate the challenge.
  - We should look into specifically this: Do we need to initialize the challenge as soon we create it? Should there be like a grace period? A time before the first upload or something? 
    - We do not need a specific hash, necessarily, we can just use the unique name we give for each challenge's folders and such... HOWEVER, we need to make sure of two things
      - 1. Integration tests do not ever make folders that overlap with existing ones (might happen since we reinitilize sqlx databases when testing)
      - 2. That students cannot access each other's data... This may be difficult without something like uuids or hashes, since I don't know if you can give specific acecss to one s3 bucket at a time.     
      - We might wanna create one IAM role, and then use session policies. Each user will then have to have their session policy updated every 12 hours.