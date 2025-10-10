# P-Pipe development log

A short detailing important decisions, thoughts and ideas collected during the work on P-Pipe. 

Not guaranteed to be comprehensive in any way. Not guaranteed to contain all working hours and/or days.


## 21-08-2025

### 1 - project management

Thought I had this idea for a logbook before, turns out I never got around to it, or never bothered to upload it to the repo, either way, this starts it.

Decided to create a GitHub projects project to track development. Didn't really know which template to get, or what to use, so just chose "feature release" feels kinda silly to work in an "issues" kinda manner when I'm just one guy. Then again, my attention span, and my memory are both so short-term, that I'm basically a new person every time I open up the project, so it kinda makes sense...

Using issues to indicate new features is kind of strange though, even though it allows me to tag each new feature with whatever it should be tagged with.

Entered "create Rust backend" as the first feature to be implemented

Also entered task requiring creating a "project plan", whatever that is, DTU requires that it is handed in by the 24th of September.

Also entered task to create somewhat of a timeline for the project, so I can have some deadlines to promptly ignore. 

Think I need a stronger way to link the project to the repository itself... or do I? I don't really know...

Discovered I had a previous project for this "PP-Pipe" which was meant to be the "Preliminary" version of P-Pipe. No work in there, think I'm gonna delete it. It's private, so who really cares?

The Data Dispatcher (Double-D (wink wink (5th grade humor))), is pretty clear in how I'll implement it, though I should still *write some technical documentation for the DD* regarding how it works, what it should do, so on and so forth. 

I've thought a bit about how users'll actually provide their "pipelines" an important thing to remember, is that while the DD might test one's data ingestion capabilities, it doesn't really require a "pipeline" per say... Maybe a few important steps is to:

- Define what actually defines a "pipeline" in the sense of the project
- Define the MVP and the "optimal product" in regards to features and capabilites
- Find a cool sounding name - like the dispatcher has

Spoke to Julian in Aalborg. He recommended providing users with a docker container and having them develop their entire pipeline in there. That way, they can easily send their entire pipeline back and forth, and there are few restrictions on what can be put into the pipeline. Moreover, the container can come pre-packaged with example "proctor" commands. So however the server will prompt the pipeline for stuff like retrains or predictions, can be made transparent to the user. Of course, trusting and running a devcontainer that "some guy" made is a pretty huge security risk, though I'm sure for this project, we can ignore that, and in the future, we can circumvent that. 

### 2 - Rust backend

Started work on the rust backend. Came up with question of giw are we going to run Python files? (And how much are we going to rely on running Python files, as opposed to just pure Rust??) One can of course also question: "Why Rust??"... I don't quite know, maybe the backend would be better if it were pure Python... Will think about it.

#### Pyo3 issues:

Had some problems with PyO3. I now remember how much I despised it when working with it before. It also does not answer the age-old (for me) question of *Statically linked or Dynamically linked Python*? Given that this is something that'll run outside of a user's hands, we can probably rely on Dynamically linked Python, though.

Encountered the old error of "not able to run, python13.lib.so.1.0 not found". Made an issue on this. Have had the issue before. Was solved by changing virtual environment Python to 3.12. Suspicious, since 3.12 appears to be the Python version of the base conda environment (does Pyo3 get its version from here somehow, or is it just a conincendece?). 

Running command `ldconfig -p | grep libpython` (reccomended [here](https://github.com/PyO3/pyo3/issues/4813)), yields `libpython3.12.so.1.0 (libc6,x86-64) => /lib/x86_64-linux-gnu/libpython3.12.so.1.0`, even before changing virtualenv Python version to 3.12. Likely that `libpython3.12.so.1.0` is some kind of linux-dependent python "compiler/interpreter" which is either not present in miniconda installations, or outside of where pyo3 looks.   

**Should perhaps start debugging like (Factorio) Kovarex? Everytime there is issue, make test recreating issue, then you have test for posterity?**

#### UV?

In line with the thoughts about Python and Miniconda before, some people had similar issues, which were then solved by using UV. Perhaps I should switch to UV, it would fit with the project...


#### Seperate dev_logs?

PyO3 is already an issue that might require looking into? I have just now, briefly considered whether the dev log should be split into different subfolders? That way, each issue that requires research or the like, can have a dev log file attached to it? Does that make sense or is it insane amounts of documentation?


## 25-08-2025

Had some thoughts about the DD not really making up a whole "pipeline" - just part of it. Testing the ability to grab this data, do some feature engineering, and hand it in again, does not test the ability to create a pipeline in itself, for that we probably neeed something more sophisticated (or manual).

Decided it would be good to have an overall plan / vision about what needs to be done, what sub-parts need to be made. Tried to do this in GitHub Projects, but issues isn't really conducive to showing that some things are not "specific" issues, but more overall planning. 

Milestones, though, can be tagged unto issues, have a start and an end date, and probably fit the bill quite nicely.

Though having a timeline in GitHub Projects appears to be a shitshow, even so, it would be a shame not to have everything in one place.

## 05-09-2025

Made Python work in Rust with PyO3. Did by fixing two issues: One with linking cc compiler (done by removing extension-module (should only be chosen if you make python-extensions)). 

The other issue was fixing the libpython.3.12.1.so file not being found. As it turns out, these are shared files, not kept by the current virtual environment. Fixed by setting LD_LIBRARY_PATH to the place where it keeps those .so files. Usually this would be in usr/lib or some other place there. Miniconda appears keeps them to itself if it is the first one to install that particular version. 

Switched to using uv. Know, that uv still uses some miniconda Python files, as this appears to be the easist for it. It does not appear to cause issues, just be aware that even though we set the PYO3 venv with `export PYO3_PYTHON=/home/cicero/ppipe/.venv/bin/python`, it does not set to find the .so file from that same location.

The aboev export PYO3_PYTHON command does make sure the PYO3 interpreter uses the installed UV packages, this was tested and verified.

## 08-09-2025

Worked on getting dispatcher module to work in Rust, two interesting errors:
1. Could not find dispatcher module before explicitly adding current folder to PythonPath with `sys.path.insert(0, os.getcwd())`, suggested solution was also adding folder to PythonPath directly: `PYTHONPATH=$(pwd) cargo run`. 
2. Couldn't find click (wonder why?) Initially, it was not installed. Installed it with `uv add click`, did not fix issue. Set PyO3 interpreter with `export PYO3_PYTHON=/home/cicero/ppipe/.venv/bin/python`, still did not fix it. Only after activating uv venv directly with `source .venv/bin/activate` did work. Suspect this means that `PyO3_PYTHON` variable, does not work as expected...

Made decision that rust should not only import modules and such to run code, they should run no code at all, only modules. This makes the whole process of executing the dispatchers, etc. a matter of rust running the "runners". Made such a mockup runner script for the file_splitter and the dispatcher. Successfully ran it.

Now running into a cryptic issue when importing the dispatcher (which uses pandas): "C extension: pickle not built. If you want to import pandas from the source directory, you may need to run 'python setup.py build_ext' to build the C extensions first." Not a lot of information online, seems to be an error with the pandas installation. Most people suggest reinstalling pandas. I suspect it is a problem from using LD_LIBRARY_PATH, as whatever C extensions that pickle uses comes from somewhere outside of the LD_LIBRARY_PATH

Considering not using Rust, right now might be more trouble than it is worth...
That is not even to mention what might happen when we start using more "unique" packages than Pandas...
Right now, solution appears to just be to go with Rust spawning a process for each Python file that needs to run, and then calling each Python script that needs to be called...
...That just requires that each Python tool has a CLI tool made with it, which shouldn't *really* be an issue ig.


# 09-09-2025

Documented decision to trash pyo3. Started cleaning up repo
Going to begin work on using the Python files through CLI commands.
Also preparing for meeting with Nicki. Will try to get an idea on:
- What should be the scope of the project? What is likely I *can* make
- What is best tofocus on from a feasability / importance standpoint? The data module? Or the continuous integration module? 
- What should I do in regards to ancillary studies? Should I ask people on opinions and stuff?
- How important is a frontend, do you think?

For tests:
- need to remove fixtures in the tests for the dispatcher, not necessary
- Need to add note if (non integration) tests of dispatchers are run without the "splits" directory exists, they will fail if so

Apparently `__init__.py` is required as a file in the tests directory. Wonder why? Thought they removed that functionality / necessity

We need to add plenty of debugging and logging information to Python scripts. Remember, stderr and stdout are the only ways that Rust are informed of the failures / successes of the script! Otherwise, it just gets exit codes!

# 19-09-2025

Completed project plan, sent off to Nicki. Said we wrote a bit "personally", was expected. Going to change that for the report, surely.

Worked on backend stuff in Rust mostly, didn't touch Python
(Unlogged) Had previously decided on, and completed table_initialization.rs for challenges, transactions, and completed_transactions for Postgres
The table structure set up here, had to be changed somewhat

Started by creating standard GET, POST, DELETE endpoints for Challenges, easily done, had small issues with DOUBLE PRECISION being float8, as opposed to integer8
Postgres type naming convention is shit...
Tested these endpoints real quick in Postman, appeard to work, still need to write tests for them

Next, worked on transactions and adding these. They should be generated automatically from challenge options. 
Wrote code reasonably quick, ran into quite a few issues, mostly considering Rust -> Postgres type conversions.
Once more had to restructure what types everything was in the tables, but ran into one big issue:
Postgres INT4RANGE has no straightforward Rust representation. This is shit.
Tried writing a custom struct, which is essentially a wrapper on std::opt::Range in Rust which implements ToSql and FromSql (thanks, copilot)
Did not work, however. Don't exactly know why, whenever I used it, it did not seralize correctly, in the sense that whenever I tried to execute queries with those structs included as parameters, the parameters did not correctly replace the string places... Best idea for why this is, is because I did not implement traits for multiple inserts at once (no idea how I'd even do that, tbh). Overall, the juice didn't seem to be worth the squeeze.
Copilot recommended using JSONB as the rows_to_push column for postgres, that way, I could push a Vec<Range<i32>>, or a Vec<(i32, i32)>, but this did not work out straightforward like, since it would have to be pushed through some serde::to_value or serde::to_string bullshit, which didn't work.
Made me think, however, why do we even need something like Vec<(i32, i32)>? We only need *one* range per transaction (ideally), so having rows_to_push be a Vec<i32> or in postgres INTEGER[] would be adequate. 
One issue here, is that postgres now does no checking that what we push is actually a range... We might push more numbers, which will then fuck up during deserialization or subsequent handling... however, none of that is user-side, so we might be able to account for it, we'll just have to be more vigilant. 
Another issue issue is, this *might* create issues if we want AI storytellers or other things to push different datapoints, like randomized... We might want to do that. In this case, however, we could simply shuffle the data beforehand...
Right now, we only have 2 values in our rows_to_push to imitate a range, a *start* and a *stop* (inclusive, exclusive), this can be changed to include specific indices to push. Would make storage requirements a bit higher, but might be worth it to push a non-contiguous range of data points at a time. IF we go for this, it also makes the postgres type make a bit more sense... nice.

Didn't really have all that much time to clean everything up.

A lot of the issues that I had today with types and rust to postgres and whatnot, *COULD* be solved by an ORM like diesel or sqlx... I just really don't like adding layers of abstraction. Give it a few more issues, however, and I might fold, right now, most of my time is spent fixing that sort of issues, and last time I worked with Postgres was no different... 

Got a ton of TODO's that I really need to look at at some point, really before I clean it up.

Next time, will clean up a bit more, work on creating tests for all endpoints with reasonable parameters


# 23-09-2025:
Worked on migrating to sqlx, worked well enough, there were some hiccups though.
There is a bit of unknown stuff about query! (macro in sqlx), since it works by performing compile-time checking. Which is nice.
However, that requires an active connection to the database, something that *might* be set through an environment variable DATABASE_URL, but I'm not sure.
It complained a lot in the beginning, then not really, even though I'm doing things that should make it angry... Not sure why it tolerates that
    - Basically, a lot of the issues were with non nullable columns in Postgres becoming Option<T> in Rust, it didn't like that one bit...
    -But it works

Encountered some problems trying to get table_initialization to actually work. Don't really know how modules work together in Rust... Would have to make something run on startup of the backend, and with that, I might as well wait until I learn how to do migration stuff. That's what they did in the Rocket example

Right now the "unit integration tests" (they're not really testing all that much, so unit test feels more correct), expect a clean database.
So I need to set up a testing database, one that I won't feel bad deleting everything from every time it spins up for testing.
Might do that through rocket.toml? Maybe.

Also need to consider if it really is good to "chain" my tests together? What if the create challenges test fails to delete stuff afterwards, and then the get_challenges fails afterwards because getting from empty db should yield empty list? 
Have to look into sqlx (or pure postgres stuff) for rolling back all operations, so a test can clean up after itself... 

# Late at night 25-09-2025

I feel terrible, everything's shit. Yet finally, some good news!
To my dismay, sqlx does not have a good way of undoing stuff on the database inbetween tests... so that's shit. 
Not unless you wrap everything in transactions, and fat chance I'm about to do that!
Soooooo I tried getting into sqlx::test, at the idea of [some guy on Reddit](https://www.reddit.com/r/rust/comments/xv55pq/rocket_how_to_clean_database_between_tests/)

That was a whole new can of worms, but it did appear to have something I knew! Tests that auto-populates arguments to test functions (fixtures in Pytest, anyone?)

In the case of sqlx it appeared to be some kind of connection pool, which is kind of an issue, because I just use a Db fairing struct from rocket like a monkey. 
Took me a little while to make the rocket client connect to that pool automatically, but it finally worked... and then nothing.

It appears that whatever pool the sqlx thing made, was not the one rocket connects to... sigh. So I looked at [this guy's blog, specifically about testing sqlx with Rocket and postgres](https://wtjungle.com/blog/integration-testing-rocket-sqlx/) (how nicely specific to my issues).

There, I found out that rocket::build() is just a convenience wrapper for creating a rocket::facet() which is kinda like a connection string (far closer to what I'm used to with postgres_tokio, I guess). From there, I thought I could just pirate the code from [that guy's github repository](https://github.com/madoke/configmonkey/blob/main/tests/common/mod.rs), but no. When I did that, my tests ran, sucessfully and whatnot, but whenever I did a test, whatever I pushed, also found its way to the "prod" database. FUUUUUUUCK

Anyways, did some back and forth, printed some values (old school, shitty debugging), and finally found out, it was because I had mispelled a variable name:

`#[database("postgres_db")]` here, postgres_db should match `rocket::Config::figment().merge(("databases", map!["postgres_db" => db_config]));`... ok. So I did that

And it returns another error! Hallelujah!

That error, I do know what is though. It is a 500 internal server error, and from the postgres output (convenient that it is "hosted" in the same docker instance, but on a different, artificial database), I coudl see that it was because the post command I was testing, simply could not find a "challenges" database. Know what that means?

I'M DOING DATABASE MIGRATIONS NEXT TIME! Yipeeeeeeeeeeeeee! Anyways, I already got the code for it in my neglected `table_initialization.rs` script. So I just need to attach that to the test_postgres thingy thing (I'm tired), and then bob's your uncle.

Tomorrow, I drink many beers, not to celebrate, but just becuase. Good job, Karl. Why thank you Karl, it has been an honor working with you!


# 29-09-2025

Did a bunch of cool stuff.

Finally got Rust backend tests to run. Didn't really require much finagling from what I did last time.
Did learn I needed to createa a Pool<Postgres> in my tests if I wanna do some sql queries to check the work of endpoints.
Also did migrations. Now the ol' table_initialization.rs will live out the rest of its days as a .sql script. RIP I guess...
Although, I found out, that migrations can only be done a single folder at a time. So one folder needs to do everything you need. Therefore, should probably split up CREATE TABLE challenges, and CREATE TABLE transactions, and whatnot to multiple files, especially if I start creating indices and enums and whatnot...
Decided to clean up files and repo and whatnot, will do more cleaning afterwards - probably gonna add all common parts into a common.rs file or smth, for test cases specifically.
Wrote tests for POST challenges 
Wrote unit testing for create_transactions_from_challenge
Also did *prop testing* for this. Prop testing is this really cool thing, where it'll create properties to try and make your test fail. You test by property ranges instead of specific cases. Nice. Gotta have me some more of those.
Also decided to rewrite how main.rs creates a rocket. It is no longer with `Rocket::build()`, decided to re-use `rocket::from_config(rocket::Config::figment())`, since this is the way I do it in tests, and I wanna be (I guess [idiomatic](https://www.google.com/search?client=firefox-b-d&channel=entpr&q=idiomatic%20meaning)?) in that way.

Did some cleaning up of Git branches and whatnot VERY carefully, didn't wanna fuck anything up.
Probably gonna be some dumbass commits I need to cherry-pick in the future
Need to take a backup of the repo every now and then... just in case. 

Gonna start work on drive data dispatcher next time, writing tests for that and all... Should be significantly easier than Rust backend... SHOULD be...

Also moved around the uv .venv, which made uv act up going "I can't find the fucking .venv file!", fixed it by running `export VIRTUAL_ENV=$(pwd)/.venv`, probably a stupid fix, probably won't last. Might need to redo uv environment (from .lock file, so easy?), although I have no idea where uv stores the current directory's VIRTUAL_ENV variable?

UPDATE: Right now, it appears uv doesn't have any active VIRTUAL_ENV variable, yet I can still run `uv add pytest` from ppipe/dispatcher/, but not from ppipe/, so it may be fixed? Who knows!

# 30-09-2025

Solid day, lotta stuff got done. Should have written during the day. Oh well.

Worked on drive data dispatcher from scratch. 

Took a little work and finagling to get it to work with Google drive. Since you have to set up API access and such. 
Created a new project in google Cloud so I don't use mlops project.
Set billing limit to 200 kr (playing it safe)
Created credentials file (that will most certainly not get pushed to Github)
The credentials can be used for oauth2 login. From here, was also possbile to create a token file so as long as we have that, and it is not expired, we can keep using it.
Should set up some headless way of doing it. There is something something about authorized users and whatnot, didn't look into it.
Bundled the whole drive access thing to two functions and one file.
oauth2 also requires that you use chrome, otherwise doesn't work (fucking shit)

Worked on data dispatcher.
Discovered that drive doesn't care about folder names, but goes by folder_ids. Might cause problems
Considered making challenges intended_data_location create some kind of uuid_extension so we're sure there is no overlap between them.
    This is also good for testing
There may be a workaround by switching up the backend.
For now, when the dispathcher finds an already existing folder, it just assumes it can safely dispatch to that folder.
Otherwise, it'll create the folder and update permissions.
But should probably separate logic of **initialization of challenges** from **dispatching data to challenges**. In Drive its forgiving, elsewhere it may not be.
If two or more folders exist, it fails, however (on purpose, nice).
Uploads as an io bistream thingy, but that is so we don't need to save a local copy of .csv and whatnot when we upload.

Thought about getting multiple fetchers, may be redudant, we can just force users to upload to a specific location, shouldn't be a problem.

Decided on making a python orchestraot which deconstructs a transaction JSON, finds and uses uses the correct fetcher, finds the correct dispatcher and calls it.
This isn't exactly in line with what we thought that we coudl just take a failed transaction and add it straight to unit testing the dispathcer, that way ensuring us against regressions.
But as long as we keep a working orchestrator (through good unit testing), shoudln't be a problem.
Also allows us to standardize, and add more logic to how we handle arguments in the JSON. 
Also requires we only write one CLI
Simplifies Rust code (only has to call one CLI)
Simplifies integration tests (only one CLI)
Does add *some* redundant behavior in how it always looks for the same arguments to unpack, we may be able to handle this by having more optional arguments on the db-side of things.
Right now, doesn't work too well with passing arguments around as **kwargs, *args and whatnot. Thought I had something good going and then went away from it to go for something simpler.
It uses a "JSON unpacker function", which may seem redundant or stupid, but I think it is the right decision, since all problems with missing or extra args or kwargs, will come as errors from that funciton, rather than somewhere else. Should make testing easier?
Will really have to talk to Nicki about this. Decided to schedule next meeting soon.
Christian might also have some ideas. He mentioned typed dicts, dataclasses, and his preference for the former. I lean more towards dataclasses now, tbh.

UV environment started acting up (couldn't find click installation)
Did all kinds of shit to try to fix it
Was only fixed by moving .venv out into its original location (fine you win, you petulant computer-child)

Almost pushed secrets to Github (twice) (shitty .gitignore after moving stuff around)
Github didn't wanna do it, something something "GITHUB PUSH PROTECTION"
No idea I'd enabled that. Made an issue to find out when/how it got enabled and how to configure it.
In same vein: Also made issue to start using pre-commit. Probably should have from beginning, tbh.

Next time, will work on cleaning, writing unittests and integration tests. 
Then will work on what to show Nicki for next meeting, as well as what questions we may have.
Then will work on more unittesting and integration testing for backend module.
Merge drive branch into main and such...
**Uninstall chrome**

All code works. Have made it a point to have each commit work code-wise. 
Christian mentioned something something makes regression to another branch automatic-ish testing possible and good. Ask him about it

Still have 21 weeks and we have a solid framework
Nice. Good job
We're on this

# 07-10-2025

Checked out [rclone](https://rclone.org/). SInce it promises to trivialize all work we otherwise would do on the dispatcher...
Had some difficulty after setting up the first remote. Apparently you should set scope only to `drive`, otherwise, it'll push files to a location you cannot see as a user.
Also, it appears you cannot push files to the root folder of your drive (probably because the file already exists there, omg)
(Yeap, because there was already a file called iris.csv there, rclone doesn't wanna overwrite in this case...)

Thought a bit about it, rclone provides pretty much provider-agnostic commands to **upload** the data, albeit with tiny differences:
...Mega does not allow file creation
...S3 does not allow bucket creation, supports versioning, etc.
...Drive has the whole AppFile issue going for it.

That means, if I choose to go ahead with Rclone, I'll still need to do **some** implementation work, having like specific ways to transfer a specific transaction to a specific provider. But I can pretty much do away with all smaller functinons like drive_folder_creation, drive_data_dispatch, etc.

Decided to throw that off for now, and work on Rust background polling...
Regarding that, I looked into postgres has like a LISTEN/NOTIFY system, which can be combined with `pg_cron` - essentially a cronjob package for a postgres database
... but this is a lot of work, and I don't wanna run into an sqlx::test issue again. So I'm gonna go with:
- Spawn an async thread in Rust
- Ping the database each $n$ seconds to see if any jobs are past their due date
- If so, run the job.

And that is it.

Spent some time working on the task scheduler for Rust
Ran into slight problems when I tried to run an sqlx reset command...
We've changed the migration script, but sqlx remembers what it ran, so it didn't wanna do it, and we have to run a `sqlx database reset` to get it to stop acting up
And that destroyed everything, and we don't care really.


# 9-10-2025

Made small changes to have attaching the scheduler be part of the .env file configuration
Should make testing easier

# 10-10-2025

Cleaned up dispatcher.rs and main.rs scripts.
Added some proper error handling to process transaction part of dispatcher
Added some more error handling to create_transactions_from_challenge in the case of missing challenge_id 

Most importantly, had a brainstorm on how we can change table structures to make more sense, especially given that we should have multiple different locations to dispatch data to. Found out a few things:
- We don't have any way to configure *where* a challenge has to dispatch data to right now
- We don't have any way of differentiating actual data upload transactions from just book-keeping or configuration transactions (like if we have to create folders, update permissions, etc.)
- We don't have any way to write who should have permissions to a given location (mail for drive, aws account for s3, so on)

Ended up deciding that we add a kind of `dispatch_to` column to challenges, which is a list of strings `TEXT[]`. This can then hold all the specific places we can dispatch to. EDIT: Should be a custom enum (places we can dispatch to, instead). Initial implementation is then just having the create_transactions_from_challenge split the data randomly bewteen the two places... probably the best.

Following that, the place it dispatches to is just challenge_id_challenge_name (challenge name being an auto-generated UUID if it is None)
a drive folder called challenge_id_challenge_name/ if drive, a folder in a bucket called challenge_id_challenge_name/challenge_id_challenge_name if s3.

The data_intended_location for transactions then simply needs to add /release_name to each release (can be as simple as adding transaction_id as the release_name), and either rclone_drive_remote_name or rclone_s3_remote_name in front for either drive or s3

Added access_bindings, a jsonb column to all tables. For challenges, it should be information for who should have access to all dispatches_to locations, like:

  '[
    { "service": "google_drive", "identity": "alice@example.com", "identity_type": "email" },
    { "service": "aws_s3", "identity": "123456789012", "identity_type": "account_id" }
  ]'

For tranasctions, it is something that'll update the permissions of the folder in question if necessary. This permissions update I'm thinking we do whenever either both or one of rows_to_push or source_data_location is NULL. That way we can differentiate between "I wanna upload some data"-transactions, and "I just wanna update permissions"-transactions

Might need to add a create_folder kind of subtransaction? Don't know how it works on s3...

Have postponed users manually entering where and when they want the specific transactions to a later time. Will probably be done by having like an additional jsonb added to the challenges table, and then unpacking this when we create the transactions.

Obviously, all of this also requires that we change some Rust code, and some Python code, so that'll be for next time, probably wednesday 15/10/2025. After this, we should be able to:

- Add challenges, which automatically adds transactions
- Automatically run those transactions in Python in the background, which dispatchse to two locations

And this is essentially our MVP. So I *think* that this should be decided as kind of our 0.1.0 version, and from there we can then create automatic testing, pre-commit, and all teh other devops stuff that we want (which at this point should be in our backlog) to ensure we have a place to reference and roll back to in case shit hits the fan. This point about deciding on what the 0.1.0 release will contain, is a point in our issues, however, something we'll decide with Nicki also, so I guess we'll hear his side of it also...

Nice. We're getting there, after this, I think we're much more golden than we otherwise would be.

**This is week 7/25, we have 19 weeks, 6 days left. We are $28\%$ through with the project time-wise. Is that good? We should ask Nicki...**