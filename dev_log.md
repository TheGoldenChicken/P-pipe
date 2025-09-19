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
