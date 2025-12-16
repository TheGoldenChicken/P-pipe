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


# 15-10-2025

The dev log from *10-10-2025* saved me. Couldn't find a template for the access bindings, so I did..., nice.

Ok, we ended up fighting with a lot of annoying things this time...
Started out nicely by splitting up and redoing the migraiton scripts... easy.
Then went on to reset the postgres server with `sqlx database reset`, works, easy no problemo.

THEN, we go about rewriting the backend to properly accept the new enums we have made.
Fuck
Shit
Balls

Start out by adding access_bindings (what type should that even be?) in Rust. Decide on an Enum of structs, nice.
Challenge then has a `Vec<access_bindings>`, good shit.
After this, create enum for dispatches_to. Good shit, no worries. We've done this before in postgres_tokio, how bad can it be?
Challenges then has `Vec<DispatchTarget>` from the postgres dispatches_to[] type.
We have to play around a bit with the traits of this, but overall it works out. 

Then, we run into the issue: error: unsupported type dispatches_to of column #8 ("DispatchTarget")

We find out quickly, this stems from query_as! and the ludicrously hard restrictions it places on the compile-time checked types. Moreover, it doesn't properly use the From<T> trait when using query_as! or query! this appears from the following threads: [thread one](https://github.com/launchbadge/sqlx/issues/1004) (this was where the solution was found), and [thread two](https://github.com/launchbadge/sqlx/issues/514) (this is where the more general nature of the error is described).

I tried a bunch to get it to work, but constantly was hamstrung by a bunch of small issues: Forgetting to write dispatches to as `Vec<DispatchTarget>`, error, forget to include access_bindings correctly, error, etc. 

End up trying to just have the postgres type be text and have the rust type be an enum. Copilot came with two faulty solutions here:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[sqlx(transparent)] // Treat as a wrapper over TEXT
pub enum DispatchTarget {
    #[serde(rename = "s3")]
    S3,
    #[serde(rename = "drive")]
    Drive,
}
```

```rust
#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "text")] // optional, for clarity
#[serde(rename_all = "lowercase")]
pub enum DispatchTarget {
    S3,
    Drive,
}
```

The former didn't work because you cannot use sqlx(transparent) in this way (it is meant for Rust structs -> Postgres TEXT as far as I know), the latter didn't work because who the fuck knows? Something about not implementing From<()>, it was akin to, but not the exact same as the original issue.

I end up cutting down all other columns, and making a test database with only the `dispatches_to` enum as a type, and then working on a dummy insert function in Rust, so it ONLY inserts to that one column, but still as a dispatches to column. Here, I can then try the solutions from the [github thread](https://github.com/launchbadge/sqlx/issues/1004), the ones that solve it are essentially incorporating this:

INSERT statements
```rust
sqlx::query_as!(
        TestChallenge,
        r#"
        INSERT INTO test_table
        (dispatches_to)
        VALUES ($1)
        "#,
        challenge.dispatches_to.clone() as _,
    )
```

SELECT statements
```rust
    sqlx::query_as!(
        TestChallenge,
        r#"
        SELECT dispatches_to as "dispatches_to: DispatchTarget" FROM test_table
        "#
    )
```

So, the `as _` and  `as "dispatches_to: DispatchTarget"` seem to be the fix.

I have already tested without either of these, and they appear to be the one line that make or break this whole machinery. 

If I had to guess, I think the error is correctly something about `query_as!` and `query!` not correctly making use of or doing stuff with the `From<T>` trait that sqlx has. This would otherwise be responsible for turning something from one Rust type to a corresponding Postgres type (I think). I don't know how it would look runtime-checked instead, something about having to use `.bind()` instead, but I don't even know if it would throw any error to begin with, since `query` the non-macro version might use `From<T>` correctly and whatnot. 

Then again, almost every issue caught with the macro at compile-time, is probably an issue we don't have to find at runtime instead, which is way slower to search through. So I guess in the end it's fine. At some point though, we may have to rip off the band-aid and stop using `query!` macros... there may be more flexibility in runtime checking, and I don't know how many issues it will present with proper testing. The main concern is the time taken to make the change, the sunk cost already, and the potentially higher development time, really I don't think the runtime issues are anything to be concerned with, application isn't big enough to run into esoteric as such issues.

Next time, we'll stay on that testing path, and try to expand it to `Vec<DispatchTarget>` in Rust to `dispatches_to[]` in Postgres. Then we'll see if it works. After that, we can make a similar development test for access_bindings and see if that works, and before you know it we'll have both....

Or we'll be dead, who knows!

UPDATE: Made it work. Pretty simple, with `SELECT` it was all a matter of

```rust
TestChallenge,
r#"
SELECT dispatches_to as "dispatches_to: Vec<DispatchTarget>" FROM test_table
"#
```

For insert, it gave a more esoteric issue about not implementing `PgHasArrayType`, that might be a missing feature on the side of sqlx... The fix was pretty simple enough, and found from [this guy panicking on reddit](https://www.reddit.com/r/rust/comments/u0mewy/help_wanted_insert_vecenum_with_sqlx_in_a_postgres/), and then [linking to this comment from the same issue from before](https://github.com/launchbadge/sqlx/issues/1004#issuecomment-1019438437). It just requires you implement this trait for the `DispatchTarget`

```rust
impl PgHasArrayType for DispatchTarget {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("dispatch_target[]")
    }
}
```

Tested to see if `[]` are necessary (github comment says otherwise), yes they are.

Really, the fact that we had to implement this pretty stupid and useless trait with only type info, tells me more that this is just something that comes from `query!` being overzealous in what'll fail on the side of Postgres. It is also supported by [some guys talking about a const_panic value](https://github.com/launchbadge/sqlx/issues/514#issuecomment-657723036), indicating it is just an issue of panicking when it shouldn't...


# 16-10-2025

Found the real error that was holding me up so long yesterday. After I tried `"INSERT" challenge.dispatches_to as _`, initially, it still didn't work. It was only when I made the whole testing function that it did. 
Now, when I tried to expand that to the whole `Challenge` struct, it gave me an error again... but there should be no differences between them right? WRONG. The initial function had this `RETURNING *`... You cannot `SELECT *`, when you need to select `dispatches_to` like: `dispatches_to as "dispatches_to: Vec<DispatchTarget>"`.

THAT is the heart of the issues yesterday. Simply doing away with `*`, and explicitly adding column names to get in `RETURNING`, fixed the issue. Nice.

After fixing that, had a few headaches trying to get `access_bindings` to play nicely. Ended up working *fairly* easily, by simply using this type for both Challenges, Transactions and CompletedTransactions: `Option<DbJson<Vec<AccessBinding>>>`, which corresponds to `access_bindings jsonB` in Postgres. From what I have tested, it appears to serialize well one way to Postgres, but I need to be sure that it serializes the other way FROM postgres. Though usually, if it passes the compiler, sqlx doesn't let me down. 

There is a slight issue, and that is the `DbJson` type, technically `sqlx::types::Json` (shares name with `serde::Json`, hence the alias). In my API, I really wanna treat it as just `Vec<AccessBinding>`, but can't as long as it is wrapped in `JsonDb`. We can easily unwrap it with something like `c.access_bindings.map(|json| json.0)`, but this might become cluttered and tedious. Copilot suggested making a wrapper function, so having like an API-side challenges and a postgres-side challenges, and have a `From` trait to change between the two. Needless obfuscation, abstraction, whatever. Briefly considered it, frfr, but then remembered that most Challenges API-side, also need to go to Postgres side at some point, or vice versa, so no point in changing to types that are incompatible with each other. Will probably just end up using the aforementioned line.

Also had some slight hiccups on having removed `NOT NULL` fields in Postgres, which automatically corresponds to having to have an `Option<T>` field in Rust. Fucked with me a bit, not much.

By the end, had all endpoints working 'as they should', that means, after a very surface-level Postman test. I don't know if all types are the right types and whatnot, need to test that explicitly next time. Speaking of which, all tests are broken, need to rewrite them, and write new ones, yay!

Also, `transaction_scheduler`, `transactions_from_challenge`, and those things, while they *compile* (because I made some compatiblity things), they don't do what they're suposed to. So I can't close the issue of rewriting the stuff in Rust... not yet anyways.  

Also need to add functionality to move attemped transactions to `completed_transactions`, this needs to be done as part of the same issue, I'd say. At this point, it's just the final Rust touch-ups before the 0.1.0 version.

Might wanna look into new way of having priorities in Github, right now, I can kinda get it to work (all having same priority, I can just reorder them), but it doesn't show a nice story of what was high- and low-priority at different times. 

This has been quite fast and informative. Also ended up using `transaction_status` as an enum, since what I learned from `dispatches_to`, challengees-side, so that's nice!


# 27-10-2025

Worked a bit on making backend work for new implementation of challenges and transactions. No troubles really.

Added `dispatch_location` column to transactions, reckon that's easier to go by, than splitting `data_intended_location` on keywords.

Right now, we just get a number of random places to dispatch to for each slice of data. That can be changed. Each slice of data should maybe also be split between different locations.
Whatever, can do later. Didn't take too long to implement what was implemented. ~1 hour.

Started writing tests, but then realized I need to account for the randomness... Shit. How the hell do I unit test that? Might be I'll have to seed the thing? But then how will I know the outcome, and will I have to recheck it every single time?
Best way to test is probably to avoid the randomness alltogether, and then test the randomness by proptesting or something?

Maybe...


# 31-10-2025

Looked into adding s3 buckets through aws, pretty simple with boto3, once I got past the initial hiccups of keys and secrets...

Turns out rclone can do the same thing with the `mkdir` command. Copilot and the internet lied to me (imagine).

Compared `rclone-python` with `rclone`. Both appear to be the same - a wrapper around the rclone CLI. Both need it installed. `rclone-python` appears better. Will go with that.

Wanted to find out more about individual remotes, for example what type they are, no funcitonality in rclone-python for that. Someone suggested a sort of [monkey-patch](https://en.wikipedia.org/wiki/Monkey_patch)

```python
def type_remotes(remote_name):
    command = "config show " + remote_name
    stdout, _ = utils.run_rclone_cmd(command)
    
    process_out = stdout.split('\n')
    for str_out in process_out:
        str_out = str_out.lower()
        if str_out.find("type = ")==0:
            str_out = str_out.replace("type = ","")
            return str_out
    return ""
```

Tried it with `rclone.type_remotes(rclone.get_remotes()[0])` as an example. It works well enough. Will go forward with that. 

Made the following notes for what I wanna make today:

1. [x] Find way of listing rclone remotes (python)
2. [x] Make kinda switch function that returns correct rclone remote based on names (or types of remotes if that is possible)
3. [x] Use this with rlone sync or copy or clone (find out the correct one)
4. [ ] Make auxilliary rclone.mkdir function to run before the other ones
5. [ ] Make rclone change permissions function

Will get on it.

Got a little stuck on number 3 (gonna stick with copy, since sync is a bit more like Github-ish operations). Basically, can't get it to work for IOBytes, which is shit. Now, I have to save the part of the file I wanna upload, and upload this, removing or keeping the old file. Sigh. But, that may be better, since I guess we kinda want the 'correct' data that we uploaded somewhere saved to a location... 

Can look more into how to do tbis in a smart way, but for now, we'll just save the part files that we're uploading somewhere right before we uplaod them. For that, we might need an extra function like

1. [ ] Make general `save_preprocessed_data` function, that is able to save a wide array of different types of functions in ways that then make sense to upload
2. [ ] Make general `load_data` function that is able to load a bunch of different formats (not just csvs!) to upload and select the proper rows and whatnot...

Alright, got Rclone uploading to s3 working now, will make into functions with more support

Halfway through implementing this, and it occured to me that having `data_intended_location` as the sole proprietor of where a file will end up, is not a good idea. Added `data_intended_name` as simply the name of the file that'll be added. This way, `data_intended_location` can aid the python file in creating a folder for the local files to keep track of what is uploaded. 

It is a bit of superfluous information, since each transaction within a single challenge, will share folder name, but I really don't think that is that much of an issue.

So I got a barebones version working. `orchestrator_cli` now works so that I can either upload to drive or s3 by seamlessly changing a single value. That is pretty baller if I do say so myself.

For next time, will work on the following:
TODO: Currently missing functionality
1. Giving other users permission to drive and s3 buckets
  - May be particularly difficult for drive folders, since they require that we know their ID (fixable by adding UUIDs)
   - *will* require us to go outside rclone functionality, RIP
2. Initializing folders, s3 buckets and whatnot should be trivial with rclone.mkdir()
3. validating that a transaction is formatted correctly, (contains correct values and whatnot)
4. Adding UUIDs as part of each challenge and/or transaction
5. Adding generalized data readers for multiple file types (fetchers)
6. adding generalized data savers, so these read files can also be subdivided and locally saved in a proper manner for multiple file types (savers)

## Over ALL!:
- Should work this first working version into the transaction scheduler, just so we can say we have done it.

Overall, this is good progress, we have used $36\%$ of our time, and yet the first-ish version is already complete. That is nice. 


# 03/11/2025

Started working on the transaction scheduler as planned, it occured to me, that whenever I had the backend running in the background, the scheduler was actually running as well (I never turned it off in `.env`). Does this mean it just worked? Partly yes! When pushing to drive, there were no issues, but then immediately afterwards, when pushing to AWS, it ran into issues, simply because it cannot create the folder first. 'No problems' I thought, that should just return an error.

But the tokio loop silently terminates when an error is returned. I had no idea of this. The fix appears simple enough, handle the error, and use `continue`. I'll try that.

The problem really was that the error was propagated by `?`, which meant it went to tokio instead of being handled inside of the function. An error in tokio means the thread terminates.

There were some minor hiccups (forgot to await on the transaction scheduler running). Also forgot `async move` Previously, it was this:

```rust
tokio::spawn(transaction_scheduler(pool.clone()));
```

Which, I'm not entirely sure of, but I can only think that it didn't properly spawn a new async thread. 

Briefly considered (and also did), order transactions returned by `transaction_scheduler` by `scheduled_time`, that way, we can add functionality to just always have transactions at `scheduled_time`$=0$ be those that create folders.

But now, I'm thinking that it's way easier to do simply add it to create_challenges. Or as a sub-part of that. Something that runs as a kind of if-statement. Either that, or we're gonna have to add another table for kinda `auxilliary requests` on top of the `judgement requests` table we already have to add...


# 04/11/2025

So I guess I got lost in the sauce, because while I was implementing a `rclone_make_dirs` function, I tested the regular `rclone copy` with s3, and it works... without any issues. Weird.

Spent like an hour trying to find the issue, before I realized, I'd silently changed underscores to not exist. So underscores were still being passed from Rust to the orchestrator, which was making S3 pushes fail when done from Rust, but not from the terminal. Jesus. Making a dead-simple sanitizer script fixed this rather minor, but supremely annoying issue...

Fucked around some with adding a functionality for the script to remove completed transactions from transactions and add them to completed_transactions.

The whole thing appears to work as it should now... So the testing begins.

Next time, gonna have to split up dispatcher.rs into more manageble pieces, right now, it's pretty bigg

# 9-10/11/2025

Wrote some simple integration tests for the challenges. No biggie there. 

Then, ran into a big problem; testing transactions being added... No fucking way I'm doing the old thing of creating a long list of valid transactions, and then testing if the output matches the input. Very brittle, very stupid.

Proptesting is obvious... Thought first about what I wanted to proptest, and then thought that I could just test stuff like: "If a challenge has this many release_proportions, will it correctly upload transactions to the database?". Makes sense, and can be expanded to a bunch of other shenanigans... Started doing it, then ran into the problem that proptest!{} does not work with async. Fuck. Tried doing a block_on, with tokio, but didn't work, that fucked with the underlying sqlx async runtime, leading to a panic (not from sqlx, mind you, from the underlying Rust runtime). Tried both `proptest-async` and `test-strategy`, the latter of which touts a macro, kinda like proptest, [that works with async](https://docs.rs/test-strategy/latest/test_strategy/#proptestasync--). Problem is, any argument "passed" (are property-based arguments *passed* to a function?), needs to implement the `arbitrary` trait, which `PgConnectOptions` and `PgPoolOptions` does not. No fucking way I'm doing that, phat chance.

Final thoughts then, are that proptesting while doing integration testing may be stupid...

Therefore, came to my senses and came up with the idea that **integration tests should only test, what concerns the integration itself** (quote me). Meaning, the ability to create a bunch of logical transactions from a single challenge - "WGAS?". The integration test should only be the last part: "Can we conncet to the db, and upload said transactions to it?". Meaning, we basically only need to test the `add_transactions_into_db` function's ability to execute and add what it has, which is technically in `challenges.rs`. We can make it somewhat robust, by having a set challenge, creating the transactions using the regular function, and then the test criteria is being able to deserialize the postgres transactions to sane and working rust transactions. That should do it. Everything else, we can get in unit tests and proptests. Meaning the ability to not go above 1, the ability to create $n$ transactions for $n$ `release_proportions`, etc., we'll figure that out when we get there. 

Next time, therefore, we write some unit tests for transactions, and then a dead-simple integration test for the test.

Today's lesson: Tests should be simple; INTEGRATION TESTS SHOULD BE SIMPLERERERE!

I can already imagine myself writing 'E2E tests should be simplesssssssst' in the near future. Let's wait and see.

Also, found some good material on [advanced rust testing, particularly in databases](https://rust-exercises.com/advanced-testing/06_database_isolation/01_sqlx_test.html), should read it given the time.

# 14/11/2025

Worked on aforementioned integration tests and proptests for backend in Rust. Started out creating unittests in `challenges.rs`, mostly focused on proptesting `transactions_from_challenge` and `add_transactions_to_db`. The latter I had to do integration tests for, but the former I could do with proptesting to test various things.

I ended up also making some integration tests for transactions, for GET, DESTROY, DELETE... All pass except GET, which fails roughly 50% of the time. I don't know why exactly, but I suspect it is because of the transaction scheduler, I don't know though, will have to test more. 

Now I just need to add testing to the Python part (I'll probably wait with E2E testing, though), and I'll be done with the first release. Next milestone will then be either:

- Judgement module
- Multimodal data support
- Active Learning support

I'll probably make estimates for what all three will require and take it from there. For now, we're in a pretty good spot.


# 18/11/2025

- Made small fix to rust tests, made is so `scheduler_fairing` is only attached if there is not a test running. Imagine I can manually attach the scheduler fairing in the case I need to test it.
  - Reminds me: **I should test the scheduler**
- Wrote rust 'script' which saves rust instances from `testing_common/instances.rs` to .json files so Python can load them in a more genuine manner
- Wrote some tests for `py_modules/orchestrator.py` - Ensures it works for both drive and s3
- Didn't clean it up
- Knocked out the 'decide on common test directory'-issue. Just treated `py_modules` as a module that `py_modules/tests` can import from... So far works, but it might fuck with debugger or running from vs_code, guess I'll see, didn't check it
- Removed 'graveyard', as there was some old pytests in there that were fucking up the test results whenever I ran `pytest`.
- Decided on using a parameterizered fairing to run tests for multiple cases of dispatcher (s3, drive, etc.)
  - Needed to manually make these difference transaction instances with both s3 and drive, might need to change this in the aforementioned `save_instances.rs`
- Found the difference between `rclone-python.purge` and `rclone-python.delete`, the former deletes ALL including the folder it created, the latter just deletes the content
- Found out that `rclone-python.rclone.purge` just deletes drive files, they're still in my trash... This **might** create problems in the future, for now I dunno...

## Judgement Module

Module kinda like current `transactions.rs` and `challenges.rs`, should handle sending requests to students, asking them to confirm data points or make requests for inference.

### MVP points:

- Extra table and schema for requests that hold information on what students should "give" in regards to data. 
- Endpoint for students to find all relevant requests for them
- Endpoint or other method for students to submit answers for requests
- Integration with scheduler to automatically send or make available the requests at their allotted time
- 

### Extra points

- Security for endpoints to ensure students can call requests endpoints, but not challenges and transactions endpoints
- Way for students to upload answers to batch predictions
- Easy way for students to set up their own endpoint server to flip the script for batch predictions

### Requires
1. Making Rust schema for requests
2. Making postgres table for requests
   1. Possibly making postgres table for request answers
   2. Possibly also making postgres table for completed requests
3. Making admin endpoints for requests: GET, POST, DELETE, PUT
4. Making user endpoints for endpoints: GET, POST
 Implementing 4ish endpoints
5. Making Python script to evaluate user POSTs or scheduled read from requests 
    1. Possibly making possible to download files from certain folders
    2. Keep track of statistics like time-to-completion or correctedness of answers.


## Multimodal data support

Expand current .csv file support to allow for multiple different files. Current idea is one .csv file, where each row has an extra column that points to other files, most likely images.

### MVP Points

- Users should either be able to specify, or the program should automatically detect a "multimodal data"-column indicator thingy
- When detected as existing, each data fraction upload should also include all relevant "extra" data, that is the images and whatnot.
- 

### Extra points:

- Intentional corruption of data should be able to affect the multimodalities of the data, so filters on images and whatnot.
- If Judgement is implemented with requests, should also be able to request the multimodal data
- If active learning is implemented, users should also be able to implement the multimodal data

### Requires

1. Deciding on whether we should change the challenges schema to allow for a "multimodal" column specification, or whether we should just go with a standard column name for multimodal data.
   1. Likely better with the former, as we can more easily expand this to include multiple sources of multimodal data.
2. Changing the challenges schema in all sources (including unit-tests and whatnot), to include new mulitmodal data column name
3. Change transactions schema in all sources (including unit tests) to include new multimodal data column name (cannot reference challenges, transactions must be 'atomic')
4. Change functions such as `create_transactions_from_challenge` to include multimodal data
5. Change `upload_with_rclone` and possibly `orchestrator` in `py_modules` to account for multimodal data.
6. Writing new integration tests that ensure the multimodal data is pushed as it should

## Active learning module

Implement previously discussed possibility for user to request access to certain parts of data, y-values, certain columns, etc. Given that it requires students to interface with the API, there may be some overlap with the judgement module... 

### MVP Points:

- Include possibility in challenges to specify 'extra' columns, that are not pushed except if explicitly requested
  - Also include possibility to specify cost of these points
- Include field in challenges that tracks 'resources' used by students
- Have endpoints for students to request the extra data

### Extra Points:

- Allow for cost of data points to be dynamic depending on class or similar


### Requires:

1. Changing transactions to include fields that specify which columns to push or which *not* to push
2. Changing `py_modules` to only push fields as specified by new transaction fields
3. Creating relevant endpoints, GET, POST, etc for students to use
4. Create a special flavor of transactions, or change transactions to only push requested columns
5. Creating integration and unit tests for the whole thing


## Decision
- Likely start with multimodality, seems easiest and most juice for the squeeze
- Then go on with judgement
- Then finally active learning (seems difficult and not muchg ain from it...)

# 26/11 - 01/12-2025

Had a meeting with Nicki, mostly about report structure. In general, it will follow this overall structure:

- Abstract
- Introduction
- Theory / Methodology
  - ML, MLOps, DevOps, MLOps Maturity Model, etc.
  - All overall subjects used (testing, but not pytesting, backends, but not rocket, etc.)
- Project studies
  - Quantitative and qualitative studies
- Solution ideas and proposals
- P-pipe introduction
  - "Demo" of desired product
  - State of current product
  - Overview of modules (should probably just be a flowchart looking thing)
- Development "Story"
  - Primer: Overall tools used
  - 'Ramble' about tool choices dependent on experience rather than merit
  - Development of, and choices made in various versions

- Discussion / Possible improvements
  - What could be done differently?
  - Did very structured way of working make sense?

- Conclusion
- References
- Appendix

Overall, Nicki emphasized angling the report as more of an experience piece / guide - a kind of collection of experiences for future implementors.. He also emphasized the importance of ensuring people know what kind of report it is in the beginning - it can be annoying to read halfway through a report only to figure out it ain't that kind fo report by the end.

About the questionaires and interviews and such, he didn't appear to put much importance. They could back up the project, but the act of implementing something appeared to be important enough, even *if* it turns out it doesn't make sense to make - then I've just made something cool that doesn't do much for the good of man ig.

In Nickis opinion, the most important part is simply to get writing on many of the methodology and "story" parts as possible - we can always change it later. To that end, I've started writing the development of the first MVP version, splitting it somewhat up into "Database", "Backend", "Dispatcher", and "Testing". 

I've given up on trying to make it perfectly in chronological order - That doesn't make sense really. A lot of images, code snippets, links, and such, I don't include yet, I'll make later. I also have the idea to either link in the appendix, or through references, to my own Github issues descrbing the work done on various parts of the project. That seems like a very descriptive way of doing things.

A lot of the explanation now seems to focused around "this tool or that tool", which as far as I understood from Nicki, appeared a good way to explain one's choices, and even so, the choice of writing specific code snippets is not really important. I might need a more structured way of making these comparisons, right now it seems fairly...random.

Finally, Nicki suggested making the judgement module over the multimodal data support. His argument being that it will more than the other parts, "finish" the program. Moreover, feedback is pretty important. I overall agree, and then again, it doesn't seem like ALL that much work to implement it. I've made the subtasks as Github issues, and right now, it seeems like more of the same, so we just have to add an extra two tables, with Rust schemas, some endpoints, and some additions to the automatic scheduler.

I decided that students will answer requests for data and predictions by way of calling a PUT endpoint. This way, they can just upload their answer as an attached .json, which the server can validate in a minimal way. In the same manner, the PUT endpoint can be made so the request is automatically scheduled to be validated, moved to completed_requests, etc. 

# 04/12/2025

Started work on judgement module, meaning issues 57-60. Simply made one table meant to hold requests, and have a second "on standby" for completed_requests. 

Had a bit of back-and-forth with myself and copilot, about the best way to structure asks for requests (repqest_payload) and the general structure expected of users for the request response (expected_response)... jsonB needs to serialize into a specific Rust type, so I had to make a type for that. However, if it is a jsonB, that is a struct in Rust, it *cannot* be an enum in Postgres, meaning we lose some validation on the side of Postgres, but I think that is worth doing still.

Ended up creating a `RequestType` enum that is one of three "payload" structs: `DataValidation`, `BatchPrediction`, and `CalculatedFeature`. I figured the following:

In general, data points should be expected in like a json-type format, ala

```
type_of_request: {
    Type: "BatchPrediction", <- Automatically assigned during serializing
    "items": [
        {
            "col1": 69,
            "col2": "Value string",
            "col3": False
        }
        {
            ....
        }
    ],
    "count": 3
}
```

- DataValidation:
Payload (items) is a vector of ints: `[2, 6, 42, 59, 69]`, which is just the data points that need to be returned
Expected response is a `BatchPrediction`-type response (reused for now, but might need to make cursory differences between them)

- BatchPrediction
Payload (items) is as `BatchPrediction`
Expected response is probably like `DataValidationPayload`, although this only supports integer predictions, so might need to make a specific one that has, like:

```
expected_response: {
    Type: "PredictionRespnose",
    "items": [
        {
            "row":, 69,
            "prediction": 55,
            "prediction_2": False,
            "prediction_3": "Derpherp derp derp derp"
        }
        {
            ....
        }
    ],
    "count": 3
}
```

- Calculated Feature
Payload is either like payo (if we assume they have already recieved data), or explicitly like BatchPrediction, if we wanna help them along... In either case, it should contain two additional values: `feature_name`, so they know how to submit it, and `feature_information` so they know how to calculate it.
Expected response probably like Batch prediction, as long as it contains the specific new columns, we don't really care about what extra columns they submit...


Initially, I wanted to have two different columns for requests, one for the request type and one for the payload. In either case, the request type would be an enum in Rust, and request_payload could not be an enum in Postgres (since it should be a struct in Rust). request_type can be implicitly contained in request payload, so we didn't really need to include that.

Made simple endpoints for GET requests, POST, DELETE, DESTROY requests, really much easier when we have made it the first time... Only issues came from aforementioned differences between `DbJson` and "Serde" `Json` - easily fixed.

Need to add some type of validation when submitting a request so they don't submit shid requests

Next time, will start work on more user-facing endpoints, that should only be able to return requests for one's specific challenge and whatnot. At this point, we might wanna consider adding UUIDs to challenges, otherwise people pirating each other's requests and POSTs will be a real issue (integer IDs are easy to figure out).

Need to remember to add warning in report that allowing users to upload jsons is a security risk, SQL injection and whatnot. Don't know if there is a real risk, but then again, I'm not a security expert...

Also need to complete implementation for CompletedRequest, both Database-side and Rust-side.

We also briefly considered, that we need to standardize plural vs singular definitions in our code. Sometimes its called "challengeS", other times "challenge", same for "requestS" vs "request", etc. Minor thing, but really annoying, especially if you come from the outside and are not me...

Finally, considered  if it made sense to give like an API to students to make it easier to interface with the specifics of the platform, so they won't have to spend time structuring their pandas arrays and such in the specific json formats... that is kind of a foregone conclusion (does that make sense even?). Could also include basic functionality to connect to, and send the endpoint requests... though that might also just be a simple iteration on: "read the fucking documentation for whatever fastAPI-clone you wanna make, probably just use requests, dummy!"

Only other thing to consider with requests, is we have to use an enum to control what *types* are accepted in the submitted jsons (which are seen as `Hashmaps<String, Serde::Value>)` in Rust.)


# 05/12/2025

Did some work on Requests from the user-end. Decided that the GET api/requests/challenge_id should get all requests for a single challenge_id (we'll change all ids to UUIDs later), but that the eventual PUT should work only on single requests at a time (that means the id of the specific request (later uuid))...

We somewhat finished making the PUT, first does some checking to see if the `response` `RequestType` matches what it should... We were about to make a function for this, but then realized we can just match it up against the `RequestType` of `expected_response`... that of course assumes the teacher has made it correctly, or that we've validated when the request has been made or something... We do this comparison in a bit of a weird way using `mem::discriminant`, we'll need to look into that more, also, we use `into_inner()` another place we also need to look more into (thanks, AI!).

Anyways, then we are suposed to run some Python code to validate the data itself... We could possibly do this in Rust, after all, the comparisons are rather simple, but in the case of partial comparisons, within bounds and such, I'd rather we isolate that to Python, as we usually do, it is the domain of data... **One downside of this**, is that the `RequestStatus`, which will be determined from this Python script, will have to be... returned from the Python script, meaning we must somehow be able to return this from the CLI, which we have mentioned before, is cancer. We'll have to find some holistic way of doing this... parsing strings perhaps? After all, we can kinda limit what comes out into the `stout` of the Python process pretty heavily. Either that, or we can keep it in `stderr` as information, potentially having it be even closer managed. Or perhaps (don't know if this is possible), we can do it simply through the CLI itself... perhaps Click has something...

Anyways, after that, we create the `CompletedRequest` (which we also implemented, and had minor problems with `RequestStatus` not being `Option<T>` in Rust, but being optional on the db). Finally insert this into the `completed_requests` in database, and at the same time, delete the entry from `requests`... This last part I say "at the same time", because it actually was pretty simple to make transactions work in sqlx, so that it won't execute either transaction unless both succeed. I should probably use this for transactions as well... A shame it took me so long to figure out that it is dog-easy to make it work.  
<<<<<<< HEAD
=======


# 08/12/2025

Did work on the report and the judgement module... 

Report-wise, started writing on MLOps. Might have to write on ML in general beforehand. Had some formulation problems, because I really wanna specify the importance of MLOps, especially in the in the context of what Nicki likes to say: Programming is *our* (as ML engineers) laboratory, but that ultimately came off as rant-y. If anything, I think it makes more senes to reserve it for kind of a "why MLOps?" subsection... potentially not include it all. *If* it really only is methodology, we shouldn't really care about the *why* of MLOps, and mostly just about the *what*.

Overall, getting kinda stresssed (again) about the report, since I only have 42-ish percent of the entire time left. Probably shouldn't really stress, but stiiiiilll.

Went on to working on the judgement module. Overall, went pretty smoothly. The entirety of the important python part could almost be vibe-coded (hate that word). In regards to the CLI for it, tried to follow the example set by `orchestrator.py` as much as possible, though it occured to me that I have no `ochestrator_cli.py`... should maybe do that.

Also found out two things:

1. You can make enums in Python (cool)
2. You can manually specify exit codes in python (wooow)

The second point is most interesting, Python generally considers anything that is not a 0 some kind of error. 1 is typically syntax errors, 2 are usage errors, and the rest are various types of general errors, specified by the individual program. I figured, I could use this as a semi-robust way to pass information on what the result is. I opted for 0: Correct, 1 and 2 unused to make way for regular Python errors, and then the rest like `PARTIAL_CORRECT`, `INCORRECT`, etc. set on the remaining errors. 
Granted, this robustness is kind of lost when I then fail the function if I can't deserialize the `judgement_message`... but that's another story, I can remove that nazi deserialization policy. On a side-note, I also opted to require each part of a `BatchPrediction` or `DataValidation` response to include a `row` field, as otherwise the comparison would be a total shitshow to figure out... Not nice, this is a small requirement for students I think, and totally reasonable.

Finally, also added the aforementioned `judgement_message` to the output which is essentially just data on how many errors there were, what kinds, etc. Intended to give the student more detailed feedback on where they fucked up. Added it as a `sqlx::types::Json<rocket::serde::json::Value>`. Didn't wanna create a specific type for it yet (plus it isn't too important that it follows a specific format), and mixing `rocket::serde` and `sqlx::types` in this manner makes it serialize up real nice, even though it doesn't really make that much sense. Also need to add tolerance to each request (potentially unseen by user), as given by the challenge specifications.

Made a bunch of `launch.json` additions to kinda debug `judge_cli.py`, seems to work the way it should.

Did cursory tests of all endpoints using Postman... stuff looks good so far. Also added minor endpoints to GET, DELETE, DESTROY all completed requests and whatnot. There is a minor issue right now, where the id is reset when going from request -> completed_request. Meaning if we have request.id = 5, and make completed_request from it, it is set to the current serial value of completed_request. Kind of an issue, since we cannot logically connect requests to completed_requests now, but that'll sort itself out if and when we switch to using UUID's, since no chance these'll be inferred automatically (they might actually be, but we need to we aware of this!).

Next time, gonna work on automatically creating requests, either all the time (random time periods between), or easier, specifically with each transaction. In this, it also makes sense to include options for these in the challenges themselves. This might be the time to take care of [github issue #61](https://github.com/TheGoldenChicken/P-pipe/issues/61), adding a potential json column to challenges to control how requests, transactions, and so on, are made automatically by the scheduler.

Easiest, and most minimal implementation might be just to add a few `DataValidation` requests for each transaction created, do this, then create tests, **and then** take care of issue #61. Then we can have #61 as a whole new feature potentially... This might also make sense for the future implementation writing abouts, letting us create essentially a whole new chapter on that. Nice

Sent the questionaire to S/M-KID, gotten like 17 more answers, nice. Would like around 100, tho. Made some questions for potential qualitative interview for engineers and professors, will execute them if and when I have the time.

Finally, obviously also need to clean up all files, run `rustfmt` and allat, will do that when the feature release rolls around...


# 11/12/2025

Had a response from a professor who said that he'd submit to an interview next wednesday or Thursday, score. Very nice.

Worked on automatically generated requests for the judgement module today. Also, discovered a glaring issue with the way we handle `expected_response`.

Started by working on a way to include Options in `challenge`, this seemed pretty important, as there are a 100 and one ways we can add requests. For now, I added a struct in Rust, which is just a `jsonB` in Postgres, that'll hold these options: `ChallengeOptions`, basically just a bunch of `Option<T>` fields with an attached `default()` method.

Obviously, this required me to add `ChallengeOptions` to a bunch of endpoints everywhere, but this was no issue. Instead of having it be an `Option<T>`, I let serde choose its default if it wasn't given, that way I don't constantly have to check if `ChallengeOptions is Some()`, which is nice. I should maybe do this for other things, like `acesss_bindings`...

Had a small issue or thought on whether or not I should add them to transactions as well... I generally want my transactions to be more or less 'atomic', but it is a lot of duplicate areas I gotta add them, and right now, no Python script uses anything in `ChallengeOptions`. Thought that might change, however, and took it in every transaction, space on the db probably isn't an issue, and we can always discuss that at length later.

Next, worked on `sheduler.rs` mostly. Basically, based on `tx.challenge_options.makes_request_on_transaction_push`, I create requests when pushing transaction (duh). This required that I was able to generate a request from a transaction. I decided first that the `request_type` will simply be chosen randomly from tx.`challenge_options.challenge_types`, I can just sample from there. That was a bitch.

Then, I needed to generate a request, which was easy for `DataValidation` (just get a random subset of rows). This I did as a method on `DataValidationPayload`, and subsequently was reminded - I should probably move more functionality from functions to methods on schemas... might make stuff more 'encapsulated', which I want maybe? Actually also, should maybe move `request_from_transaction` to be a method on `Transaction`... the aforementioned Payload, only has `from_transaction`. (Which I also need to implement for both `CalculatedFeaturePayload` and `BatchPredictionPayload`)

But then I had to get the `expected_response`. This requires reading the data, which we decided **rust must not do!**. Fuck. Thought about it long and hard for a short while, then decided the best way is to make a Python script, that given a `request_type`, or a transaction, will generate `expected_response`, or a `request_type`. It is a **shitty** solution, because once again, we have to pass data from Python to Rust THROUGH the CLI, fuck, but it really is the best solution. When it comes down to `CalculatedFeature`, for example, no way in hell we wanna go with Rust to make the `expected_response`!

Right now, the amount of Rows to request is just random uniform based on the size of the range, but should also be an option. Also, the `expected_response` is set to placeholder, which is the same as the `type_of_request`. The final "issue" came with how I should add the request to the database itself. I wanted to reuse `endpoints::requests::add_request`, but can't since it uses `Db<Connection>`, whereas the scheduler has access to `PgPool`, which for some stupid reason, cannot be deref'd from `Db<Connection>`. Looked on Github, couldn't find anything. Tried implementing a helper function (like `add_user_to_db` and `add_user` from rustfs program), couldn't make it work, stupid. Gonna look into it later, for now I just added an `add_request_to_db_with_pool` in `scheduler.rs`, so issa good right now.

Finally, I also made everything async, of course, but then did nothing with it, since I just await everything instead. I reckon we can safely create requests while doing other stuff, so I need to start spawning some threads somewhere.

Also also, discovered an error where if the request generation fails, the whole function fails, and since the transactions have already been removed, and since the completed_transaction is only inserted following the request generation.... well the transaction is lost **I SHOULD FIX THAT!**

Also, discovered (or probably, was reminded), that the scheduler craps out entirely (meaning it doesn't resume ever!) if there are any errors with any sub-functions, since they are propagated all the way up... Should *really* address this! Can make the whole platform crap out over a single error.

Also remembered **I should make tests for the transaction scheduler!**, pretty important piece of hardware, no tests for it!

Also discovered I use milis in Rust, but Postgres was using seconds, fixed this by just multiplying `created_at` by 1000 everywhere. Nice.

Next time, should start with making the Python expected_response creator, and then, writing tests for everything. There is still a lot of work to do with the feature as a whole, but just being able to make `DataValidation` requests, only when transactions are pushed, is a good place to start.


# 16/12/2025

**The iris dataset only contains 150 rows...Bruh.**

Occassaionally (at seemingly random times), pandas will complain about not having fsppec. Think it is connected to when this is the `source_data_location` given `"s3://bucket/data.csv"`

Alter table to not require foreign key constraints!

Worked on implementing `get_expected_response` on the Python side, and then making simple tests on the Rust side. Apart from the listed above issues, no real issues came up. I had to change `instances` a bunch of times, but no biggie. For launch.json, i might wanna create a separate list of arguments that I can then reference in it. Might make more sense than filling up the lines there with arguments for the cli interfaces.

Accepted that I probably won't have functionality for `BatchPrediction` and `CalculatedFeature` yet... which is fine. For now, it just throws `NotImplementedError` in Python if the user tries to use them. This might not be handled in Rust in the best way, however. There are also a few panic cases, especially when dealing with `rows_to_push` on the transactions side... in general we just gotta watch out for this.

I had a few problems making the unit tests for Request, as I had to generate a Request, and then use the prcess_request function to see the result matched. THey didn't because of randomness. So I opted to replace `rand::rng()` with a `global_rng()` function, that returns exactly the same, but when called with a `feature`-flag (something you can do with `features --featres deterministic`, for example), it uses a seed value of 42. Nice. This is partly because rust can't just globally seed everything like what Numpy and Torch does with `set_seed` like functions.

The amount of tests I made for Requests and Scheduler was lacking a bit. The scheduler remains completely untested, so nothing there. I only test that it *Can* generate a request, since I don't really know what to compare the request generated from a transaction against... I guess a premade request? But at that point, we're back to completely brittle tests...

For request schemas, I made a simple proptest for `DataValidationPayload`, that tests that no matter the `rows_to_push` range (randomly created), the `items` returned (what the user is suposed to get), is correct, so it is contained within rows_to_push range. 

This might do it for request tests on the scheduler side, though I still need INTEGRATION TESTS for requests. This is important, right now, we're not testing the endpoints in any way.

Nor are we testing that python returns the correct `expected_result` when we call it with Rust (or at all for that matter). But here, I had to make pytests, and I wasn't feeling it. Likewise, we're not doing any testing for the judgement module as a whole. We should do that.

So **testing** is the name of the game before we can finish this... 'sprint', if you wanna call it that.