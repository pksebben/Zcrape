use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, value::Value};
use std::fs::File;
use std::io::{BufReader, Read};
mod conf;
use conf::Conf;
mod zulip_request;
use zulip_request::{Narrow, MsgRequest};
use std::collections::HashMap;
use dotenv::dotenv;
use std::env;
use std::fs;
use std::fmt;

mod db;
use db::{DB};

fn init() {
    // perform app initialization business
    println!("Initializing Zcrape scraper...");
    dotenv().ok();
    println!("dotenv loaded.\n\nenv contains:\n====================\n");
    for (k, v) in env::vars() {
	println!("{} : {}", k, v);
    }
    println!("\n\n");
}


fn main() {
    let pattern: Vec<String> = std::env::args().collect();

    for file in &pattern[1..] {
	let foo: String = fs::read_to_string(file).expect("light it up!");
	let buf = MessageBuffer::from_json_string((&foo).to_string());
	for message in buf.messages {
	    println!("===============\n{:?}", message);
	}
    }

    init();

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

    // do I want this to return the data or to manage it completely?
    // fn pull(&mut self) -> Result<String> {
    // 	// prep curl request
    // 	let mut req = MsgRequest::new(
    // 	    self.bookmark.into(),
    // 	    1,
    // 	    self.window_size.into(),
    // 	    env::var("ZAPI_EMAIL").expect("no api email in env"),
    // 	    env::var("ZAPI_PASS").expect("no api pass in env"));
    // 	req.narrow("link".to_string(), "has".to_string());
    // 	req.narrow(self.stream_id.to_string(), "stream".to_string());

    // 	let results = req.curl();

    // 	match results {
    // 	    Ok(data) => {
    // 		// save data to disk (how / where does this happen?)
    // 	    	// update bookmark
    // 		// save object state (bookmark)
    // 		// return okay signal
    // 	    }
    // 	    Err(error) => {
    // 		// retry
    // 		// log
    // 	    }
    // 	}
	
	

	// let results: String;
	// let attempts = 0;
	// while attempts < 10 {
	//     results = match req.curl() {
	// 	Ok(data) => {
	// 	    self.save();
	// 	    data
	// 	},
	// 	Err(error) => String::from("curl failed"),
	//     };
	// }

	
	// this should manage the thread that pulls messages
	// this should:
	// 1. Keep track of our place (in terms of messageid) + a list of all pulled messageids
	// 2. Use that bookmark to run the next query
	// 3. Ensure that messages were pulled correctly
	// 4. Halt on error, send a C&C signal (with forensics), and await further orders
    
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

    // filter messages out by criteria.
    fn filter(&self) {
	
    }
}
