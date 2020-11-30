# Recharvist (working title)

	This devlog is meant to track development of the knowledge persistence platform for Recurse.  It may, in many places, be somewhat scattered, as it more or less represents my train of thought while creating.  If you are here to learn the scope of the project, any details as to it's implementation or use, or to contribute, I recommend starting with the Readme, and using this document as a contextual supplement should you desire insight into the process-thus-far.  Anything documented here should be taken with a grain of salt, as the ideas contained herein are all subject to change.

# Log

## Monday, November 16, 2020

Prior to today, I build the api scraper by hobbling together a custom Curl implementation (that operates with strings and subprocess::Popen) and wiring that in to a couple of classes to manage Message data.  At the moment, the program has the ability to pull arbitrary numbers of messages, based on chunks of message ids.  

On deck:
- filtering for messages
- tagging / scoring links based on contextual heuristics
- stepping function for the puller

Imran and I collaborated on a better solution to this than I had before; because we are limited in requesting the next 'window' of message results by virtue of the fact that we rely on {last_message_id} to provide the anchor for the next request, I had thought we were going to have to do all the message pulling on a single thread.  However, we can split the results by narrowing by stream.  This will add some complexity to the program but allow us to make multiple requests.

- setting up the sqlite db
  - processing queue for messages pulled but not processed
  - ?? outbox for messages processed but not sent ?? - this may be an unnecessary step, if data integrity can be maintained without it.
  
  
In order:
- set up sqlite

## Tuesday, November 17, 2020

I'm working on getting sqlite wired in with ```rusqlite``` and I'm coming up against a problem (one that I could work around if I really wanted to by hardcoding) regarding database schema.  What I'm doing is trying to create a function that would allow the immediate translation of rust structs into database schema.  

Essentially, then, what I'm attempting to build is some format of ORM.  This is obviously a problem much larger than the scope of this project (and one that, if solved, could quickly turn into a full time job.)

It just dawned on me, writing this, that i have not yet looked up 'rust orm'.  There's a package called diesel. I'm going to go rtfm on that now.

NOTE:  I want to push this to git and share it. 

15:38:27:	
Diesel looks pretty solid.  It's going to require a little bit of tooling to set up - which means the ansible script for the server is also going to need some tooling.

..or not (this is from diesel)...
```
When preparing your app for use in production, you may want to run your migrations during the application's initialization phase. You may also want to include the migration scripts as a part of your code, to avoid having to copy them to your deployment location/image etc.

The diesel_migrations crate provides the embed_migrations! macro, allowing you to embed migration scripts in the final binary. Once your code uses it, you can simply include embedded_migrations::run(&db_conn) at the start of your main function to run migrations every time the application starts.
```
	
## Thursday, November 19, 2020

First thing today is to move everything from Config.rs to .env
Second thing is to properly .gitignore stuff and push to repo, post on checkins

15:20:46:
Have yet to post.  Thinking about ditching diesel and going back to rusqlite.  

## Friday, November 27, 2020

Up next:
- finish error handling on http methods
- get rusqlite up and running and pipe the links out to a db
# TODO

	for things that don't necessarily fit into a structural todo
	
 - Prep a test to see if multiple concurrent requests can work given zulip's api.

## Saturday, November 28, 2020

Today I am shaving some Yaks.

on stage:
Debugging!  Downloading lldb project for a backend, and going to wire it in.  Leaving these intstructions open b/c the dl is huge.
