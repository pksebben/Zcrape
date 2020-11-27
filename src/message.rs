use std::fmt;
use std::fs;

use serde::{Serialize, Deserialize};
use serde_json::{Deserializer, value::Value};

use crate::cull::Cull;

#[derive(Serialize, Deserialize)]
pub struct MessageBuffer {
    // may need https://serde.rs/impl-serialize.html
    pub messages: Vec<Message>,
}

impl MessageBuffer {
    // populate buffer from json dump of messages
    pub fn from_json_string(json: String) -> MessageBuffer {
	
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
pub struct Message {
    pub id: i32,
    pub content: String,
    pub timestamp: i32,
    pub subject: String,
    display_recipient: String,
    pub stream_id: i32,
    pub flags: Vec<String>,
    pub reactions: Vec<Reaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reaction {
    pub emoji_code: String,
    pub emoji_name: String
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
    pub fn from_json(json: &str) -> serde_json::Result<Message> {
	let m: Message = serde_json::from_str(json)?;
	Ok(m)
    }
    // constructor.  Takes an object from Deserializer from_str().into_iter()
    // you must iterate over the list of messages and pass each to this fn()
    pub fn from_obj(obj: serde_json::Value) -> serde_json::Result<Message> {
	let flagvec = &*obj.get("flags").unwrap().as_array().unwrap();
	let mut flags: Vec<String> = Vec::new();
	let mut reactions : Vec<Reaction> = Vec::new();
	println!("\nchecking emojis: ");
	for reaction in &*obj.get("reactions").unwrap().as_array().unwrap() {
	    // println!("{} : {}", reaction.get("emoji_code").unwrap().as_str().unwrap(), reaction.get("emoji_name").unwrap().as_str().unwrap());
	    reactions.push(Reaction{
		emoji_code: String::from(reaction.get("emoji_code").unwrap().as_str().unwrap()),
		emoji_name: String::from(reaction.get("emoji_name").unwrap().as_str().unwrap())
	    })
	};
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
	    reactions: Vec::new(),
	};
	Ok(m)
    }

    // to make testing easy
    pub fn pprint(&self) {
	println!("id: {}\ncontent: {}\ntimestamp: {}\nsubject: {}\ndisplay_recipient: {}\nstream_id: {}",
		 self.id,
		 self.content,
		 self.timestamp,
		 self.subject,
		 self.display_recipient,
		 self.stream_id,
	)
    }

    // TODO: this is a placeholder.
    pub fn calculate_score(&self) -> u32 {
	3
    }
    
    // TODO: This is pointless
    pub fn extract_tags(&self) -> Vec<String> {
	let mut tags: Vec<String> = vec!();
	let beginner_kwords: Vec<&str> = vec!(
	    "beginner",
	    "intro",
	    "entry-level",
	    "easy",
	    "gentle",
	    "friendly",
	);
	for keyword in beginner_kwords {
	    if self.content.contains(keyword) {
		tags.push(String::from("beginner"));
	    }
	}
	tags
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

