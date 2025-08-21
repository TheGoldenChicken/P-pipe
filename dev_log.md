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