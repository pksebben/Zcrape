# Zcrape

	A little scraper for retrieving data from a zulip server.
	
# Project Goals

What we are trying to provide, is a persistent, quality store of links to high-value resources that requires little to no active management and populates off of a message board.

The use-case that sparked the idea was the message board for the Recurse Center.  We realized that there was demand for a collection of resources (similar to a wiki) but a lack of interest in managing such a collection.  After a number of discussions, a few core features endemic to a knowledge store began to emerge:

	- Persistence
	
		Data is only useful if it remains accessible.
		
	- Discoverability
	
		Data is only useful if you know it exists.
		
	- Currency (as in, being current)
	
		Data ages, not always well.
		
	- Utility
	
		Data is only useful if it is useful, and not all data is useful all the time.
		
	- Organization
	
		Because some data is useful some of the time, it's important to be able to express *what* use it is.
		
This is a lot to try to automate.  The Zcraper project aims to (partially) address some of these concerns, whilst providing a flexible framework for users to contribute to the quality of the data provided.

There are a few assumptions that this project lives and dies on:

	- People share quality things with each other
	
		Links shared on a purpose-driven platform will, on average, be useful to those interested in the purpose in question
		
	- Links represent dense stores of value, on average
	
		Because a link points to another page, it has the potential to represent much more value than a statement of similar length on a topic.  e.g. a link to a tutorial about the python interpreter will hold more value than a ten-word insight into some part of the python interpreter.
		
	- Data need not be perfect to be valuable
	
		Some 'garbage' links will make it through, and that is okay, so long as it's the exception rather than the rule
		
	- It is possible to use automated heuristics to cull garbage
	
		Whether this can be done with sufficient fidelity to create a persistent, useful data store is sort of the learning objective of this project.  Wish us luck.
		
		
There are going to be a few components to this architecture.  Our hope is that they can be constructed and assembled in such a way as to be useful in contexts other than the one they were conceived in, but the intermediate goal is to make them useful here at Recurse.

	- The scraper
	
		This is the thing that goes through all the messages and pulls all the links.  Solves the persistence problem.
		
	- The sorter
	
		After getting the links, we must apply our heuristic model to them and cull the ones that are obviously not useful.  This is the stage where there will be 'knobs' to tweak to try and maximise the quality of the links pulled.
		
	- The bot
	
		One of the critical elements of this platform's viability is it's continued use - much of the sorting and management must be done by the users as they go through the store to find things that are relevant to them.  In order to drive discoverability, a bot that actively applies scraping and sorting to *new* posts will also post occassionally to message boards, to notify which links are making it into the library, and to serve as a gentle reminder that hey, this thing exists.  Having an active component solves one of the critical problems with wiki-like resources - that of engagement.
		
	- The database
	
		A flexible and extensible database is going to be key to the operation of the other components of this project.  Because of this, a minimally opinionated SQL database should be used.
		
	- The Library
	
		The final component is that of the user-facing site.  This both serves as an access point to the value generated, and a recursive feedback loop.  As users peruse the links in the store, they will also have the opportunity to
		
		- Arbitrarily add links and resources
		- Comment and discuss re: resource utility
		- Reorganize links so they are appropriately tagged and sorted
		- Cull garbage links that made it through the automated filter
		
		The primary design goal of this phase is to create something that is scalable, flexible, and minimally opinionated.  Users should be able to change the behavior of the platform over time to suit their needs.  Furthermore, the actual code used in this part of the production must be well commented and easy to extend and interoperate, with the hope that future generations of users can extend it's utility as much as possible. 
	
	
