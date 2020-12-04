use select::{document::Document, predicate::Name};

use crate::cull::Cull;
use crate::message::{Message, MessageBuffer};

use serde::{Deserialize, Serialize};

pub type LinkBuffer = Vec<Link>;

pub trait LinkBufferBehavior {
    fn from_msgbuffer(msgb: MessageBuffer) -> LinkBuffer;
    fn url_list(&self) -> Vec<String>;
}

fn msgbuf_to_linkbuf(mbuf: MessageBuffer) -> LinkBuffer {
    let mut linkbuffer = LinkBuffer::new();
    for message in mbuf.messages {
        // let linkscore = 0;
        // let mut linkflags: Vec<String> = Vec::new();
        Document::from(message.content.as_str())
            .find(Name("a"))
            .filter_map(|n| n.attr("href"))
            .for_each(|x| {
                linkbuffer.push(Link::from_message(&message, x.to_string()));
            });
    }
    linkbuffer
}

impl LinkBufferBehavior for LinkBuffer {
    fn from_msgbuffer(msgb: MessageBuffer) -> LinkBuffer {
        msgbuf_to_linkbuf(msgb)
    }

    fn url_list(&self) -> Vec<String> {
        let mut url_list: Vec<String> = Vec::new();
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
        self.sort_unstable_by(|a, b| a.url.partial_cmp(&b.url).unwrap());
        self.dedup_by(|a, b| a.url.eq(&b.url));
    }
}

pub trait PrintAll {
    fn printme(&self);
}

impl PrintAll for LinkBuffer {
    fn printme(&self) {
        for link in self {
            println!(
                "Link: {}\nM_ID: {}\nTags: {:?}\n",
                link.url, link.message_id, link.tags
            );
        }
        println!("printed {} links", self.len());
    }
}

// This is the format we eventually want Links to live in.
pub struct Link {
    pub url: String,
    pub domain: String,
    pub stream_id: u32,
    pub relevance_score: u32, // how are we going to calculate this?
    pub tags: Tags,
    pub message_id: u32,
}

impl Link {
    pub fn from_message(msg: &Message, link: String) -> Link {
	// let mut t = Tags::new();
	// t.add_vec(msg.extract_tags());
        Link {
            url: link.to_string(),
            domain: extract_domain(link),
            stream_id: msg.stream_id as u32,
            relevance_score: msg.calculate_score(), // how are we going to calculate this?
            tags: msg.extract_tags(),
            message_id: msg.id as u32,
        }
    }

    pub fn printme(&self) {
	println!("url:\n{}
domain:\n{}
stream_id:\n{}
relevance_score\n{}
tags:\n{}
message_id:\n{}",
		 self.url,
		 self.domain,
		 self.stream_id,
		 self.relevance_score,
		 self.tags.as_string(),
		 self.message_id,
	)
    }
}

fn extract_domain(link: String) -> String {
    let s: String;
    println!("{}", &link);
    if link.len() < 5 {
	String::from("empty url")
    } else {
	if link[..5].contains("https") {
            s = link.strip_prefix("https://").unwrap().to_string();
	} else if link[..5].contains("http") {
            s = link.strip_prefix("http://").unwrap().to_string();
	} else {
	    s = String::from("faulty_url");
	}
	let mut o: Vec<&str> = s.split(".").collect();
	if o[0] == "www" {
            o.remove(0);
	}
	if o.len() < 2 {
            o[0].to_string()
	} else {
            if o[1].contains("/") {
		let p: Vec<&str> = o[1].split("/").collect();
		let q: String = p[0].to_string();
		format!("{}.{}", o[0].to_string(), q)
            } else {
		format!("{}.{}", o[0].to_string(), o[1].to_string())
            }
	}
    }
}

// this is just to cast between the db (string) representation and the Vec representation.
pub type Tags = Vec<String>;

pub trait TagBehavior {
    // fn new() -> Tags;
    // fn add_vec(&mut self, v: std::option::Option<Vec<String>>);
    fn as_string(&self) -> String;
    fn from_str(string: String) -> Tags;
}

impl TagBehavior for Tags {
    // fn new() -> Tags {
    // 	vec![" ".to_string()]
    // }
    // fn add_vec(&mut self, v: std::option::Option<Vec<String>>) {
    // 	match v {
    // 	    Some(mut thing) => self.append(&mut thing),
    // 	    None => ()
    // 	}
    // }
    fn as_string(&self) -> String {
        self.join(",")
    }
    fn from_str(string: String) -> Tags {
        string.split(",").map(|x| x.to_string()).collect()
    }
}
