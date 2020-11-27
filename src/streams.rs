use std::collections::HashMap;
use serde_json::{Deserializer, value::Value};

pub type Streams = HashMap<u32,String>;

pub trait Populate {
    fn from_json(&mut self, json: String);
}

impl Populate for Streams {
    fn from_json(&mut self, json: String) {
	let stream = Deserializer::from_str(json.as_str()).into_iter::<Value>();
	for value in stream{
	    let streams = value.unwrap()
		.as_object_mut()
		.expect("could not interpret streams json")
		.remove("streams")
		.expect("no streams found in streams.json");
	
	    if let serde_json::Value::Array(s) = streams {
		for stream in s {
		    self.insert(stream.get("stream_id")
				.unwrap()
				.as_u64()
				.unwrap() as u32,

				stream.get("name")
				.unwrap()
				.to_string());
		}
	    } else {
		// TODO: This panic should not make it to production.
		// it is here to LMK when there is malformed data, which should never happen.
		panic!("streams were not an array!");
	    }
	}
	
    }
}
