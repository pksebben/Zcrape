mod conf;
mod zulip_request;
use std::os::raw::c_int;
use crate::link::LinkBufferBehavior;
use select::{document::Document, predicate::Name};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::time::{Duration, Instant};
use rusqlite::trace::config_log;

use std::sync::mpsc::channel;
use std::thread;

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

/*
Usage
cargo run <files>

Args
<files>    Any number of globbed files to comb through, in json format.  See below note regarding acquisition of those files.

Zcrape will comb through a set of link data, pulled from a Zulip stream, and cull based on a set of rules, inputting the links into a sqlite db. It requires that you grab the relevant data via a shell script (included ) first.  


*/

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
    let mut db: DB = DB::new_from_memory()?;
    db.make_all_tables()?;

    execute_on_args(&mut db)?;

    Ok(())
}

fn save_batch_unthreaded(db: &mut DB, files: &[String]) {
    for file in files {
	let file = file.to_string();
	let linkbuffer = extract_linkbuffer(file).unwrap();
	db.dump_linkbuffer(linkbuffer);
    }
}

fn save_batch_multithreaded(db: &mut DB, files: &[String]) {
    let (tx, rx) = channel::<LinkBuffer>();

    for file in files {
	let tx = tx.clone();
	let file = file.to_string();
	thread::spawn(move|| {
	    let linkbuffer = extract_linkbuffer(file).unwrap();
	    tx.send(linkbuffer)
	});
    }

    for file in files {
	let linkbuffer = rx.recv().unwrap();
	db.dump_linkbuffer(linkbuffer);
    }
}

fn execute_on_args(db: &mut DB) -> Result<(), Box<dyn std::error::Error>> {
    // This is spaghetti.  Let's reduce it some.
    let starttime = Instant::now();
    let pattern: Vec<String> = std::env::args().collect();

    let mut dupescount : HashMap<String, u32> = HashMap::new();
    let mut complist = OpenOptions::new()
        .append(true)
        .create(true)
        .read(true)
        .open("completion_list.txt")?;
    // TODO: truncate pattern[] by complist?

    // Initialize variables.  These are both for introspection
    let mut uniqueurls: Vec<String> = Vec::new();
    let mut numlink = 0;

    // save_batch_unthreaded(db, &pattern[1..]);

    let (tx, rx) = channel::<LinkBuffer>();
    for file in &pattern[1..] {
	let tx = tx.clone();
	let file = file.to_string();
	thread::spawn(move|| {  
	    let linkbuffer = extract_linkbuffer(file.to_string()).unwrap();
	    
	    tx.send(linkbuffer);
	});

    }
    for file in &pattern[1..] {
	let linkbuffer = rx.recv().unwrap();
	db.dump_linkbuffer(linkbuffer);
    }
    db.save_mem_to_disk("memlink.db");

    // Everything beyond this line (save for Ok(())) is for introspection, and has no effect on the resulting db.
    uniqueurls.sort();
    uniqueurls.dedup_by(|a, b| compare_dedup_log(&mut dupescount, a.to_string(), b.to_string()));
    for url in &uniqueurls {
        println!("{}", url);
    }
    println!("found {} unique domains", uniqueurls.len());
    println!("Unique Url counts: \n{:?}", dupescount);
    println!("out of {} total links", numlink);

    let endtime = Instant::now() - starttime;
    println!("Elapsed time: {:?}", endtime);

    Ok(())
}

fn extract_linkbuffer(file: String) -> Result<LinkBuffer, Box<dyn std::error::Error>>{
 
    // Read file into a buffer
    println!("reading file :: {} :::::::::::::::", &file);
    let foo: String = fs::read_to_string(file).expect("could not read file");
    let mut buf = MessageBuffer::from_json_string((&foo).to_string());

    // Cull known pointless links
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
    // remove links from known useless domains
    buf.cull_list(cull_rules);
    // remove relative links
    buf.keep("http");
    buf.dedupe();

    // Cast Message buffer to Link buffer
    let mut linkbuffer = LinkBuffer::from_msgbuffer(buf);
    linkbuffer.keep("http");



    
    // This is for introspection, and has no effect on the db. 
    // let urllist = linkbuffer.url_list();
    // for link in &urllist {
    //     println!("{}", link);
    //     uniqueurls.push(extract_domain(link.to_string()));
    // }
    // numlink = &numlink + urllist.len();

    // I think this should be the thing that creates the db.

    // ?? I can't remember what complist was supposed to accomplish
    // &complist.write(file.as_bytes())?;
    // println!("wrote: {}", file);   
    Ok(linkbuffer)
}

// Sure would be swell if I remembered even slightly what this pattern is supposed to accomplish
fn compare_dedup_log<T: std::cmp::PartialEq + std::cmp::Eq + std::hash::Hash>(dict: &mut HashMap<T, u32>,a: T, b: T) -> bool {
    if a == b {
	let p = dict.entry(a).or_insert(1);
	*p += 1;
	true
    } else {
	false
    }
}

// Extracts a top-level domain from a link.
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

