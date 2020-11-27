mod conf;
mod zulip_request;
use dotenv::dotenv;
use std::env;
use std::fs;
use select::{predicate::Name, document::Document};

mod cull;
use crate::cull::Cull;

mod link;
use crate::link::{Link, LinkBuffer, PrintAll};
    
mod message;
use crate::message::{MessageBuffer};

mod streams;
use crate::streams::{Streams, Populate};

mod statuscode;
use crate::statuscode::check_status_code;

use futures::executor::block_on;

fn init() {
    // perform app initialization business
    println!("Initializing Zcrape scraper...");
    dotenv().ok();
}

#[tokio::main]
async fn main() {
    block_on(test_req_statuscodes("https://google.com"));
    println!("get code:{}", block_on(get_code("https://rust-lang.github.io")));
    init();
}

async fn test_req_statuscodes( url : &str) {
    println!("{:?}", check_status_code(url).await.expect("bad status return"));
    
}

async fn get_code(url: &str) -> String {
    check_status_code(url).await.expect("somthing be wrong").to_string()
}

// This is the grabbag of junk to probe the pipeline.  Tweak at will.
fn test_message_pipeline() {
    let pattern: Vec<String> = std::env::args().collect();
    let mut streams: Streams = Streams::new();
    streams.from_json(fs::read_to_string("streams.json").expect("could not read streams.json"));
    for stream in streams {
	println!("{} : {}", stream.0, stream.1);
    }
    for file in &pattern[1..] {
	let foo: String = fs::read_to_string(file).expect("could not read file");
	let mut buf = MessageBuffer::from_json_string((&foo).to_string());
	let mut linkbuffer = LinkBuffer::new();

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
		    tags: message.extract_tags()
		}
		)
		);
	    // println!("Link: {}\n{}\n\n", linkbuffer.last().unwrap().url, message.content);

	}
	// println!("link buffer pre-cull: {}", linkbuffer.len());
	// linkbuffer.dedupe();
	// println!("link buffer post-cull: {}", linkbuffer.len());
	// linkbuffer.printme();
    }   
}

/*
TODO:
HARD PROBLEMS
[ ] Tagging and scoring.  This is actually turning out to be quite the hill to climb, and it's not immediately apparent whether it's even feasible without some much more complex implementation.
SOFT(ER) PROBLEMS
[ ] Get a database up and running, and port all the things to that db
[ ] Refactor so it's not "main.rspaghetti"
[ ] 404, 504 checking

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
For-now data organization
Considering that it may be impossible to effectively apply granular value metrics to links, how can we structure things such that we get a reasonable organization to the links supplied?

Sorted by stream
Stream
  blogs
  stackoverflow
  github
  

If we assume a robust mechanic for end-user sorting, we may not need such granular filtering.

Is there a way to allow end-users to *code* their way into better sets of data?

What would we need to implement for that?
consider:
Link{
messageid,
streamid,
url,

}
*/

