use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, value::Value};
use std::fs::File;
use std::io::{BufReader, Read};
mod conf;
use conf::Conf;
mod zulip_request;
use zulip_request::{MsgRequest};
use std::collections::HashMap;
use dotenv::dotenv;
use std::env;
use std::fs;
use std::fmt;
use regex::Regex;
use select::{predicate::Name, document::Document};


mod db;
use db::{DB};

/*
TODO:
[ ] create a hashtable for stream:streamid
[ ] how are we going to cull?  More importantly, *when* are we going to cull?
*/


/*
CULLING RULES
get rid of it:
- duplicates
- zulipchat.com
- 404
- twitter / fb
- bitbucket / gist ??
- repl.it
 */

/*
Tagging and scoring:
What are the ways in which we can enrich the data in-place?
- can we identify a blog post?
- how do we identify beginner resources?
- is alexa rank useful here?
*/

/*
Procedural processing

We need to cull, sort, tag, and do various other things to the links gathered.  The order of operations is important.

1. Pull links
 - we're already doing this.  One of the things we must figure out is what to do with multiple-link posts.
2. Cull 'cheap' rules
 - some rules are easier / cheaper to check against.  An example of an 'expensive' rule is 404 - which is going to require a request to be checked. (the same goes for other reponse-code based rules, like 504)

What we probably want to do, is to have <Link> objects populate with a pointer to the message (messageid would work fine.)  then cull the links, then use the messageid to populate the link.

*/


fn init() {
    // perform app initialization business
    println!("Initializing Zcrape scraper...");
    dotenv().ok();
}

fn main() {
    let pattern: Vec<String> = std::env::args().collect();

    for file in &pattern[1..] {
	let foo: String = fs::read_to_string(file).expect("could not read file");
	let mut buf = MessageBuffer::from_json_string((&foo).to_string());
	let mut linkbuffer = LinkBuffer::new();

	// these are keywords that may indicate a positive signal
	let score_rules : Vec<&str> = vec!(
	    "maintainer",
	    
	);

	// these are the rules we're going to remove elements by.
	let cull_rules : Vec<&str> = vec!(
	    ".gif",
	    ".png",
	    "gist.github.com",
	    "twitter.com",
	    "amazon.com",
	    "recurse.com",
	    "mailto:",
	    "zulip.com",
	    ".jpg",
	    "repl.it",
	    "facebook.com"
	);

	// these are culling rules that I'm not sure should be here
	let potential_cull_rules : Vec<&str> = vec! (
	    "imgur.com",
	);

	
	// remove links from known useless domains
	buf.cull_list(cull_rules);
	// remove relative links
	buf.keep("http");

	buf.dedupe();

	for message in buf.messages {

	    let mut linkscore = 0;
	    let mut linkflags : Vec<String> = Vec::new();
	    Document::from(message.content.as_str())
		.find(Name("a"))
		.filter_map(|n| n.attr("href"))
		.for_each(|x| linkbuffer.push(Link{
		    url: String::from(x),
		    message_id: message.id as u32,
		    stream_id: message.stream_id as u32,
		    relevance_score: 0,
		    tags: Vec::new()
		}
		)
		);
	    println!("Link: {}\n{}\n\n", linkbuffer.last().unwrap().url, message.content);

	}
	println!("link buffer pre-cull: {}", linkbuffer.len());
	linkbuffer.dedupe();
	println!("link buffer post-cull: {}", linkbuffer.len());
	linkbuffer.printme();
    }
    init();
}

trait PrintAll {
    fn printme(&self);
}

impl PrintAll for LinkBuffer {
    fn printme(&self) {
	for link in self {
	    println!("Link: {}\nM_ID: {}\n\n", link.url, link.message_id);
	}
	println!("printed {} links", self.len());
    }
}

type LinkBuffer = Vec<Link>;

trait Cull {
    fn cull(&mut self, cullstring: &str);
    fn cull_list(&mut self, culls: Vec<&str>);
    fn keep(&mut self, keepstring: &str);
    fn keep_list(&mut self, keeps: Vec<&str>);
    fn dedupe(&mut self);
}

impl Cull for LinkBuffer {
    fn cull(&mut self, cullstring: &str) {
	self.retain(|x: &Link| !x.url.contains(cullstring));
    }
    fn cull_list(&mut self, culls: Vec<&str>) {
	for cull in culls {
	    self.retain(|x: &Link| !x.url.contains(cull));
	}
    }
    fn keep(&mut self, keepstring: &str) {
	self.retain(|x: &Link| x.url.contains(keepstring));
    }
    fn keep_list(&mut self, keeps: Vec<&str>) {
	for keep in keeps {
	    self.retain(|x: &Link| x.url.contains(keep));
	}
    }
    fn dedupe(&mut self) {
	self.sort_unstable_by(|a,b | {
	    a.url.partial_cmp(&b.url).unwrap()
	});
	self.dedup_by(|a, b| {
	    a.url.eq(&b.url)
	});
    }
}

impl Cull for MessageBuffer {
    fn cull(&mut self, cullstring: &str) {
	self.messages.retain(|x: &Message| !x.content.contains(cullstring));
    }
    fn cull_list(&mut self, culls: Vec<&str>) {
	for cull in culls {
	    self.messages.retain(|x: &Message| !x.content.contains(cull));
	}
    }
    fn keep(&mut self, keepstring: &str) {
	self.messages.retain(|x: &Message| x.content.contains(keepstring));
    }
    fn keep_list(&mut self, keeps: Vec<&str>) {
	for keep in keeps {
	    self.messages.retain(|x: &Message| x.content.contains(keep));
	}
    }
    fn dedupe(&mut self) {
	self.messages.sort_unstable_by(|a,b | a.content.partial_cmp(&b.content).unwrap());
	self.messages.dedup_by(|a, b| a.content.eq(&b.content));
    }
}




// This is the format we eventually want Links to live in.
struct Link {
    url: String,
    stream_id: u32,
    relevance_score: u32, 	// how are we going to calculate this?
    tags: Vec<String>,
    message_id: u32,
}

impl Link {
    fn from_message(msg: &Message, link: String) -> Link {
	Link {
	    url: link,
	    stream_id: msg.stream_id as u32,
	    relevance_score: msg.calculate_score(), 	// how are we going to calculate this?
	    tags: msg.extract_tags() ,
	    message_id: msg.id as u32,
	}   
    }
}

fn pull_messages() -> String {
    // this should:
    // 1. query the api for messages
    // 2. convert messages into objects
    // 3. place messages in the "processing" queue
    let c = Conf::new();
    let (_u, _k) = (c.user, c.key);
    
    let mut req = MsgRequest::new(2,20,20,_u,_k);
    req.narrow("streams".to_string(), "397 Bridge".to_string());
    req.narrow("has".to_string(), "link".to_string());
    req.curl().unwrap()
}

#[derive(Serialize, Deserialize)]
struct MsgPuller {
    stream_id: u32,
    bookmark: u32,
    window_size: u32,
}

// Vestigal. Currently made unnecessary by scrape.sh
impl MsgPuller {
    // create new puller, starting at anchor 0
    fn new(stream_id: u32, window_size: u32) -> MsgPuller {
	MsgPuller {
	    stream_id: stream_id,
	    window_size: window_size,
	    bookmark: 0,
	}
    }

    // initialize puller variables from disk
    fn from_json(json: &str) -> MsgPuller {
	let m: MsgPuller = serde_json::from_str(json).expect("failure loading message procesor from json");
	m
    }

    // save position as bookmark by deserializing & writing to disk
    fn save(&self) {
	let m = serde_json::to_string(&self).expect("could not serialize message puller");
	let filename = format!("{}.msp", self.stream_id);
	fs::write(filename, m);
	// TODO: log
    }

    
}

fn pull_thread() {
    
    // this should manage the thread that pulls messages
    // this should:
    // 1. Keep track of our place (in terms of messageid) + a list of all pulled messageids
    // 2. Use that bookmark to run the next query
    // 3. Ensure that messages were pulled correctly
    // 4. Halt on error, send a C&C signal (with forensics), and await further orders
}

fn process_thread() {
    // this should manage the thread that processes messages
    // this should:
    // 1. run concurrent threads that each process sets of messages
    // 2. send a C&C signal when all messages have been processed
    // 3. listen to the empty processing qeue and process when it isn't empty
}
// may want to collapse process and push

// metrics for usefulness
// replies, reactions, thread length.
fn push_thread() {
    // this should push processed links from the outbox to an external db
    // this should:
    // 1. push link objects, with metadata
    // 2. mark link objects as sent
    // 3. check sent messageids against external db, and if they exist, delete them.
}



fn process_messages() {
    // this should take messages from the db qeue
    // 1. filter
    // 2. sort and tag
    // 3. place in an "out" queue	
}

#[derive(Serialize, Deserialize)]
struct MessageBuffer {
    // may need https://serde.rs/impl-serialize.html
    messages: Vec<Message>,
}

impl MessageBuffer {
    // populate buffer from json dump of messages
    fn from_json_string(json: String) -> MessageBuffer {
	
	let mut message_vec: Vec<Message> = Vec::new();

	let stream = Deserializer::from_str(json.as_str()).into_iter::<Value>();

	for value in stream {
	    // TODO: What is the behavior when no messages are found?
	    let messages = value.unwrap()
		.as_object_mut()
		.expect("couldnt cast dataset to object")
		.remove("messages")
		.expect("found no messages");

	    if let serde_json::Value::Array(s) = messages {
		for message in s {
		    message_vec.push(Message::from_obj(message).expect("could not construct message"));
		}
	    } else {
		// TODO: This panic should not make it to production.
		// it is here to LMK when there is malformed data, which should never happen.
		panic!("messages were not an array");
	    }
	}
	MessageBuffer{
	    messages: message_vec,
	}
    }

    fn get_highest_id(&self) -> Result<i32, MsgCacheError> {
	let mut id : i32 = 0;
	for message in &self.messages {
	    if message.id < id {
		id = message.id;
	    }
	}
	if id == 0 {
	    Err(MsgCacheError {
		details: "empty message buffer".to_string()
	    })
	} else {
	    Ok(id)
	}
    }
    
    fn get_lowest_id(&self) -> Result<i32, MsgCacheError> {
	let mut id : i32 = 99999999;
	for message in &self.messages {
	    if message.id < id {
		id = message.id;
	    }
	}
	if id == 999999999 {
	    Err(MsgCacheError {
		details: "empty message buffer".to_string()
	    })
	} else {
	    Ok(id)
	}
    }
    // dump to json
    fn store(&self) -> Result<(), Box<MsgCacheError> > {
	// this MUST encode such that an object can be deserialized
	let filename = format!("{}_{}.msp", self.get_lowest_id()?, self.get_highest_id()?);
	fs::write(filename, serde_json::to_string(&self.messages).expect("could not serialize message buffer"));
	// ATTEMPT ONE: Just let serde manage it
	Ok(())
	
	// this should push the whole message buffer into storage.
    }
}

#[derive(Debug, Clone)]
struct MsgCacheError {
    details: String,
}

impl MsgCacheError {
    fn new(msg: &str) -> MsgCacheError {
	MsgCacheError {
	    details: msg.to_string(),
	}
    }
}

impl fmt::Display for MsgCacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	write!(f, "{}", self.details)
    }
}

impl std::error::Error for MsgCacheError {
    fn description(&self) -> &str {
	&self.details
    }
}

// TODO: add fields for user information.  Ensure that we drop that data, but we may need it to process out posts by certain bots etc.
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: i32,
    content: String,
    timestamp: i32,
    subject: String,
    display_recipient: String,
    stream_id: i32,
    flags: Vec<String>,
}

// Gets is a convenience to extract stuff from Value without typing as many unwraps
trait Gets{
    fn getas_string(&self, q:&str) -> String;
    fn getas_i32(&self, q: &str) -> i32;
}

impl Gets for serde_json::Value {
    fn getas_i32(&self, q: &str) -> i32 {
	self.get(q).unwrap().as_u64().unwrap() as i32
    }
    fn getas_string(&self, q: &str) -> String {
	String::from(self.get(q).unwrap().as_str().unwrap())
    }
}

impl Message {
    // constructor.  json message must be singular, and correctly formatted.
    fn from_json(json: &str) -> serde_json::Result<Message> {
	let m: Message = serde_json::from_str(json)?;
	Ok(m)
    }
    // constructor.  Takes an object from Deserializer from_str().into_iter()
    // you must iterate over the list of messages and pass each to this fn()
    fn from_obj(obj: serde_json::Value) -> serde_json::Result<Message> {
	let flagvec = &*obj.get("flags").unwrap().as_array().unwrap();
	let mut flags: Vec<String> = Vec::new();
	for f in flagvec {
	    flags.push(String::from(f.as_str().unwrap()))
	}
	let m = Message {
	    id: obj.getas_i32("id"),
	    content: obj.getas_string("content"),
	    timestamp: obj.getas_i32("timestamp"),
	    subject: obj.getas_string("subject"),
	    display_recipient: obj.getas_string("display_recipient"),
	    stream_id: obj.getas_i32("stream_id"),
	    flags: flags,
	};
	Ok(m)
    }

    // to make testing easy
    fn pprint(&self) {
	println!("id: {}\ncontent: {}\ntimestamp: {}\nsubject: {}\ndisplay_recipient: {}\nstream_id: {}",
		 self.id,
		 self.content,
		 self.timestamp,
		 self.subject,
		 self.display_recipient,
		 self.stream_id,
	)
    }

    fn calculate_score(&self) -> u32 {
	3
    }
    
    fn extract_tags(&self) -> Vec<String> {
	vec!(String::from(""))
    }
    
}
