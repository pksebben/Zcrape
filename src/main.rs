#![allow(unused_must_use, dead_code, unused_imports, unused_variables)]
mod conf;
mod zulip_request;
use std::os::raw::c_int;
use crate::link::LinkBufferBehavior;
use select::{document::Document, predicate::Name};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use rusqlite::trace::config_log;

// use futures::future::{BoxFuture, FutureExt};
// use async_std::task;
// use tokio::runtime;

use std::sync::mpsc;
use std::thread;
// use std::time::Duration;

mod db;
use crate::db::DB;

mod cull;
use crate::cull::Cull;

mod link;
use crate::link::{Link, LinkBuffer};

mod message;
use crate::message::MessageBuffer;

// mod streams;
// use crate::streams::{Populate, Streams};

mod statuscode;
use crate::statuscode::check_status_code;

extern crate reqwest;

fn log(i: c_int, s:&str) {
    println!("sqlite error \n{}\n{}", i, s);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe
   { 
    config_log(Some(log))?;
   }
    println!("Preparing to dump json into sqlite...");
    // for each
    let mut db: DB = DB::new("links.db")?;
    db.make_all_tables()?;

    execute_on_args(&mut db)?;

    Ok(())
}

fn execute_on_args(db: &mut DB) -> Result<(), Box<dyn std::error::Error>> {
    let pattern: Vec<String> = std::env::args().collect();

    let mut dupescount : HashMap<String, u32> = HashMap::new();
    let mut complist = OpenOptions::new()
        .append(true)
        .create(true)
        .read(true)
        .open("completion_list.txt")?;
    // TODO: truncate pattern[] by list in

    let mut uniqueurls: Vec<String> = Vec::new();
    let mut numlink = 0;
    for file in &pattern[1..] {
        println!("reading file :: {} :::::::::::::::", &file);
        let foo: String = fs::read_to_string(file).expect("could not read file");
        let mut buf = MessageBuffer::from_json_string((&foo).to_string());

        // these are the rules we're going to remove elements by.
        let cull_rules: Vec<&str> = vec![
            "gist.github.com",
            "twitter.com",
            "amazon.com",
            "recurse.com",
            "mailto:",
            "zulip.com",
            "repl.it",
            "facebook.com",
	    "recurse.zulipchat.com",
	    "docs.google.com",
	    "web.skype",
	    "zoom.us",
	    "zulip.recursechat",
	    "zulipchat.com",
	    "zulip-coffee-bot",
	    "verizon.com",
        ];

        // these are culling rules that I'm not sure should be here
        let potential_cull_rules: Vec<&str> = vec!["imgur.com"];

        // remove links from known useless domains
        buf.cull_list(cull_rules);
        // remove relative links
        buf.keep("http");

        buf.dedupe();

        let mut linkbuffer = LinkBuffer::from_msgbuffer(buf);

        linkbuffer.keep("http");

        let urllist = linkbuffer.url_list();

        // db.dump_linkbuffer(linkbuffer);

        for link in &urllist {
            println!("{}", link);
            uniqueurls.push(extract_domain(link.to_string()));
        }

        numlink = &numlink + urllist.len();
        // &mut db.dump_linkbuffer(linkbuffer)?;

        // &complist.write(file.as_bytes())?;

        // println!("wrote: {}", file);
    }
    for url in &uniqueurls {
	
    }

    uniqueurls.sort();
    uniqueurls.dedup_by(|a, b| compare_dedup_log(&mut dupescount, a.to_string(), b.to_string()));

    for url in &uniqueurls {
        println!("{}", url);
    }
    
    println!("found {} unique domains", uniqueurls.len());

    println!("Unique Url counts: \n{:?}", dupescount);

    println!("out of {} total links", numlink);
    Ok(())
}

fn compare_dedup_log<T: std::cmp::PartialEq + std::cmp::Eq + std::hash::Hash>(dict: &mut HashMap<T, u32>,a: T, b: T) -> bool {
    if a == b {
	let p = dict.entry(a).or_insert(1);
	*p += 1;
	true
    } else {
	false
    }
}

fn extract_domain(link: String) -> String {
    let s: String;
    if link[..6].contains("https") {
        s = link.strip_prefix("https://").unwrap().to_string();
    } else {
        s = link.strip_prefix("http://").unwrap().to_string();
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

fn test_db_connect() {}

// I built something needlessly complex.  Let's try and detangle it

// Step 1: test_message_fn(f: fn(MessageBuffer))
// :: this actually grabs the files from args and runs the f() on all of them
// :: it grabs the messages and culls them, and then passes to f.  That's all it does.

// Step 2:
// :: f should :
// -take a MessageBuffer as an argument
// -output nothing

#[derive(Debug)]
enum LinkStatus {
    Bad(RFail),
    Good,
}
type StatusMap = HashMap<String, LinkStatus>;

// testing wrapper to pass to test fn

fn print_pmfm(mb: MessageBuffer) {
    // cast to link buffer
    let lb = LinkBuffer::from_msgbuffer(mb);
    // pull urls from link buffer

    let results = procmult_fmap(lb);
    println!("Processing failure map...");
    for r in results {
        println!("{:?}\n", r);
    }
}

fn procmult_fmap(lb: LinkBuffer) -> StatusMap {
    let urls = lb.url_list();

    let (tx, rx) = mpsc::channel();

    let mut statmap = StatusMap::new();

    for u in urls {
        let btx = mpsc::Sender::clone(&tx);
        thread::spawn(move || {
            let resolved = (u.to_string(), get_code(u.to_string(), 0));
            btx.send(resolved).unwrap();
        });
    }

    for received in rx {
        statmap.insert(received.0.to_string(), received.1);
    }
    statmap
}

// fn msgbuf_to_linkbuf(mbuf: MessageBuffer) -> LinkBuffer {
//     let mut linkbuffer = LinkBuffer::new();
//     for message in mbuf.messages {
//         // let linkscore = 0;
//         // let mut linkflags: Vec<String> = Vec::new();
//         Document::from(message.content.as_str())
//             .find(Name("a"))
//             .filter_map(|n| n.attr("href"))
//             .for_each(|x| {
//                 linkbuffer.push(Link {
//                     url: String::from(x),
//                     message_id: message.id as u32,
//                     stream_id: message.stream_id as u32,
//                     relevance_score: 0,
//                     tags: message.extract_tags(),
//                 })
//             });
//     }
//     linkbuffer
// }

// this is a wrapper to bundle particular state with failure types
#[derive(Debug)]
enum RFail {
    Req(reqwest::Error),
    Timeout,
    Status(reqwest::StatusCode),
    Conn,
    Panic(reqwest::Error),
    Ignored(reqwest::Error),
    TimedOut,
}

// this layer is only here because I couldn't figure out why Rust complained about
// reqwest::Error not having the .is_request() fn when used elsewhere.
// that said, I changed versions since....
fn rqe_handler(e: reqwest::Error) -> RFail {
    if e.is_request() {
        println!("bad request: {}", e); // DNS failures occur here.  Can we get more specific?
        RFail::Req(e)
    } else if e.is_status() {
        println!("got status: {}", e); // here we want to bubble the status code up
        RFail::Status(
            e.status()
                .expect("unknown failure in reqwest::Error::Status"),
        )
    } else if e.is_connect() {
        println!("connection err: {}", e); // here we want to fail quietly and shelve the link
        RFail::Conn
    } else if e.is_timeout() {
        println!("request timeout"); // here we want to pause execution and wait.
        RFail::Timeout
    } else if e.is_builder() {
        println!("WE'RE ON A LIIIIINK TO NOOOWHEEERE");
        RFail::Ignored(e)
    } else {
        panic!("unknown err {} on {:?}\n\n{:#?}", e, e.url(), e)
    }
}

fn get_code(url: String, timeout: u32) -> LinkStatus {
    match check_status_code(url.to_string()) {
        Ok(data) => LinkStatus::Good,
        Err(e) => {
            if e.is_request() {
                println!("bad request: {}", e); // DNS failures occur here.  Can we get more specific?
                LinkStatus::Bad(RFail::Req(e))
            } else if e.is_status() {
                println!("got status: {}", e); // here we want to bubble the status code up
                LinkStatus::Bad(RFail::Status(
                    e.status()
                        .expect("unknown failure in reqwest::Error::Status"),
                ))
            } else if e.is_connect() {
                println!("connection err: {}", e); // here we want to fail quietly and shelve the link
                LinkStatus::Bad(RFail::Conn)
            } else if e.is_timeout() {
                if timeout < 10 {
                    println!("timeout retry: {}/10", timeout);
                    get_code(url, timeout + 1)
                } else {
                    println!("request timeout"); // here we want to pause execution and wait.
                    LinkStatus::Bad(RFail::Timeout)
                }
            } else if e.is_builder() {
                println!("WE'RE ON A LIIIIINK TO NOOOWHEEERE");
                LinkStatus::Bad(RFail::Ignored(e))
            } else {
                panic!("unknown err {} on {:?}\n\n{:#?}", e, e.url(), e)
            }
        }
    }
}

// load in a message buffer and check the urls for status codes
fn test_bulk_message_status_check() {
    let pattern: Vec<String> = std::env::args().collect();
    for file in &pattern[1..] {}
}

// test functions against a culled version of live-data message buffer(s)
fn test_message_fn(f: fn(T: MessageBuffer)) {
    let pattern: Vec<String> = std::env::args().collect();
    for file in &pattern[1..] {
        let foo: String = fs::read_to_string(file).expect("could not read file");
        let mut buf = MessageBuffer::from_json_string((&foo).to_string());
        cull_msgbuf(&mut buf);
        f(buf)
    }
}

// culls in-place. Mutates.
fn cull_msgbuf(m: &mut MessageBuffer) {
    let cull_rules: Vec<&str> = vec![
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
        "facebook.com",
    ];
    m.cull_list(cull_rules);
    m.keep("http");
    m.dedupe();
}

// This is the grabbag of junk to probe the pipeline.  Tweak at will.

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

// Probably Deprecated. in favor of procmult_failmap

// async fn process_failmap(urls: Vec<String>) -> HashMap<String, LinkStatus> {
//     let mut failmap : HashMap<String, LinkStatus> = HashMap::new();
//     for url in &urls {
// 	let handle = spawn_multithread_status_check(url.to_string(), &mut failmap).await.expect("something went wrong with super_long_function_call");
//     }

//     while &urls.len() > &failmap.len() {
// 	println!("awaiting hashmap population...");
// 	async_std::task::sleep(Duration::from_secs(2));
//     }
//     failmap
// }

// Deprecated in favor of procmult_failmap

// // this is the entry point that *should* create a new thread
// fn spawn_multithread_status_check(url: String, failmap: & mut HashMap<String, LinkStatus>) {
//     // here is the place that we assign values to the failure map
//     // we need to run the query code (which is recursively async due to timeouts)...
//     thread::spawn(move || {
// 	let status: LinkStatus = get_code(url.to_string(), 0);
// 	failmap.insert(url, status);
//     });

//     // ...and push the return of that query code to the failmap
// }

// ///////////////   deprecated   ///////////////
// //////////////////////////////////////////////////////////////////////
// I want to do more work on async, because it's suuuuperrrr coooollllll
// BUT This is probably better served by being multithreaded.
//
// fn get_code(url: String, timeout: u32) -> BoxFuture<'static, LinkStatus> {
//     async move {

// 	match check_status_code(url.to_string()).await {
// 	    Ok(data) => LinkStatus::Good,
// 	    Err(e) => {
// 		if e.is_request() {
// 		    println!("bad request: {}", e); // DNS failures occur here.  Can we get more specific?
// 		    LinkStatus::Bad(RFail::Req(e))
// 		} else if e.is_status() {
// 		    println!("got status: {}", e); // here we want to bubble the status code up
// 		    LinkStatus::Bad(RFail::Status(e.status().expect("unknown failure in reqwest::Error::Status")))
// 		} else if e.is_connect() {
// 		    println!("connection err: {}", e); // here we want to fail quietly and shelve the link
// 		    LinkStatus::Bad(RFail::Conn)
// 		} else if e.is_timeout() {
// 		    if timeout < 10 {
// 			println!("timeout retry: {}/10", timeout);
// 			async_std::task::sleep(Duration::from_secs(2));
// 			get_code(url, timeout + 1).await
// 		    } else {
// 			println!("request timeout"); // here we want to pause execution and wait.
// 			LinkStatus::Bad(RFail::Timeout)
// 		    }
// 		} else if e.is_builder() {
// 		    println!("WE'RE ON A LIIIIINK TO NOOOWHEEERE");
// 		    LinkStatus::Bad(RFail::Ignored(e))
// 		}
// 		else{
// 		    panic!("unknown err {} on {:?}\n\n{:#?}", e, e.url(), e)
// 		}
// 	    }

// 	}
//     }.boxed()
// }

// I believe that this was deprecated in favor of procmult_failmap
// but was also hacked from async => multithreaded
// and might be necessary for reimplementing async

// I was tired the last time I hacked on this

// fn run_msgbuf_via_sasc(m: MessageBuffer) {
//     // translate the message buffer to Vec<url>
//     let mut urls: Vec<String> = Vec::new();
//     let linkbuffer = msgbuf_to_linkbuf(m);
//     for link in linkbuffer {
// 	urls.push(link.url.to_string());
//     }

//     // run the hashing algo against that vec
//     let x = process_failmap(urls);

//     // print it
//     println!("{:?}", block_on(x));
// }
