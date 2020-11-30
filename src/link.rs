
use crate::cull::Cull;
use crate::message::Message;

pub type LinkBuffer = Vec<Link>;


pub trait LinkBufferBehavior {
    fn url_list(&self) -> Vec<String>; 
}

impl LinkBufferBehavior for LinkBuffer {
    fn url_list(&self) -> Vec<String> {
	let mut url_list : Vec<String> = Vec::new();
	for l in self {
	    url_list.push(l.url.to_string());
	}
	url_list
    }
    
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

pub trait PrintAll {
    fn printme(&self);
}

impl PrintAll for LinkBuffer {
    fn printme(&self) {
	for link in self {
	    println!("Link: {}\nM_ID: {}\nTags: {:?}\n", link.url, link.message_id, link.tags);
	}
	println!("printed {} links", self.len());
    }
}


// This is the format we eventually want Links to live in.
pub struct Link {
    pub url: String,
    pub stream_id: u32,
    pub relevance_score: u32, 	// how are we going to calculate this?
    pub tags: Vec<String>,
    pub message_id: u32,
}

impl Link {
    pub fn from_message(msg: &Message, link: String) -> Link {
	Link {
	    url: link,
	    stream_id: msg.stream_id as u32,
	    relevance_score: msg.calculate_score(), 	// how are we going to calculate this?
	    tags: msg.extract_tags() ,
	    message_id: msg.id as u32,
	}   
    }
}

