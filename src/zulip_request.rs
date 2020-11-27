use subprocess::{Redirection, Exec};
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, value::Value};
use std::fs;
/*
Zulip request builder

This module provides an interface for Zulip's api (using curl).

In order to use narrows and other functionality, it is recommended to always declare *Request types as mutable.
*/


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

pub struct Narrow {
    operator: String,
    operand: String,
}

impl Narrow {
    pub fn new(operator: String, operand: String) -> Narrow {
	Narrow {
	    operator: operator,
	    operand: operand
	}
    }

    pub fn to_string(&self) -> String {
	format!("[{{\"operand\" : \"{OPERAND}\", \"operator\" : \"{OPERATOR}\"}}]",
		OPERAND = self.operand,
		OPERATOR = self.operator
	)
    }
}


pub struct Curler {
    command: String,
}

impl Curler {
    fn from_string(string: String) -> Curler {
	Curler {
	    command: string,
	}
    }

    fn new(opts: String, method: String, url: String) -> Curler {
	Curler {
	    command: format!("curl -{OPTS} {METHOD} -G {URL}",
			     OPTS = opts,
			     METHOD = method,
			     URL = url
	    )
	}
    }

    fn add_data_urlencode(&mut self, key: String, value: String) {
	self.command = format!("{} --data-urlencode {}=\'{}\'", self.command, key, value);
    }

    fn add_data(&mut self, key: String, value: String) {
	self.command = format!("{} -d \'{}={}\'",self.command, key, value);
    }

    fn add_user(&mut self, user: String, password: String) {
	if self.command.contains("-u") {
	    panic!("attempted to add two user fields to Curl command");
	} else {
	    self.command = format!("{} -u {}:{}",self.command, user, password);
	}
    }

    fn execute(&self) -> Result<String, subprocess::PopenError> {
	let stream = Exec::shell(&self.command)
	    .stdout(Redirection::Pipe)
	    .capture()?
	    .stdout_str();
	Ok(stream)
    }
}


// object to construct and send queries from the Zulip api get-messages
pub struct MsgRequest {
    api_user: String,
    api_key: String,
    site: String,
    narrows: Vec<Narrow>,
    anchor: u64,
    before: u64,
    after: u64,
}

impl MsgRequest {
    // TODO: complete constructor
    pub fn new(anchor: u64, before: u64, after: u64, api_user: String, api_key: String) -> MsgRequest{
	MsgRequest{
	    site: String::from("https://recurse.zulipchat.com"),
	    narrows: Vec::new(),
	    anchor: anchor,
	    before: before,
	    after: after,
	    api_user: api_user,
	    api_key: api_key,
	}
    }

    pub fn narrow(&mut self, operator: String, operand: String) {
	self.narrows.push(Narrow::new(operator, operand));
    }

    // TODO: Narrows
    // TODO: There needs to be a check that results were returned.  Curl status codes will show up green for empty sets.
    // TODO: improve narrows to be flexible and take input, like stream etc
    pub fn curl(&self) -> Result<String, subprocess::PopenError> {
	let mut c = Curler::new(String::from("sSX"),
				String::from("GET"),
				format!("{}/api/v1/messages", self.site));
	c.add_user(String::from(&self.api_user), String::from(&self.api_key));
	c.add_data("anchor".to_string(), self.anchor.to_string());
	c.add_data("num_before".to_string(), self.before.to_string());
	c.add_data("num_after".to_string(), self.after.to_string());
	    
	for narrow in &self.narrows {
	    c.add_data_urlencode(String::from("narrow"), narrow.to_string());
	}
	println!("curler command:\n{}\n\n", c.command);
	c.execute()
    }
}
