use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};
use serde::{ser, Deserialize, Serialize};
use serde_json::Value;

use crate::link::{Link, LinkBuffer, LinkBufferBehavior, TagBehavior, Tags};
use crate::message::{Message, MessageBuffer};
use std::collections::HashMap;

// fn init() {

//     let schema = ["CREATE TABLE if NOT EXISTS message (
// id INTEGER PRIMARY KEY,
// flags BLOB,
// content TEXT NOT NULL,
// timestamp INTEGER NOT NULL,
// subject TEXT,
// display_recipient TEXT,
// stream_id INTEGER,
// )",
//     "CREATE TABLE if NOT EXISTS link (
// id INTEGER PRIMARY KEY,
// domain STRING,
// stream_id INTEGER NOT NULL,
// relevance_score INTEGER,
// tags STRING,
// url STRING NOT NULL,
// message_id INTEGER NOT NULL)"];

// }

// object to handle db connections etc.
pub struct DB {
    connection_string: String,
    conn: Connection,
    schema: Vec<String>,
}

impl DB {
    pub fn new(connection_string: &str) -> Result<DB> {
        let schema = [
            // "CREATE TABLE if NOT EXISTS message (
            // id INTEGER PRIMARY KEY,
            // flags BLOB,
            // content TEXT NOT NULL,
            // timestamp INTEGER NOT NULL,
            // subject STRING,
            // display_recipient STRING,
            // stream_id INTEGER,
            // )",
            "CREATE TABLE if NOT EXISTS link (
id INTEGER PRIMARY KEY,
url TEXT NOT NULL,
domain STRING,
stream_id INTEGER NOT NULL,
relevance_score INTEGER,
tags STRING,
message_id INTEGER NOT NULL)",
        ];
        let mut db = DB {
            conn: Connection::open(&connection_string)?,
            connection_string: String::from(connection_string),
            schema: Vec::new(),
        };
        for s in &schema {
            db.add_schema(s.to_string())
        }
        Ok(db)
    }

    // this is sort of a workaround.  To keep the schema decoupled.
    pub fn add_schema(&mut self, schema: String) {
        self.schema.push(schema);
    }

    pub fn make_all_tables(&self) -> Result<()> {
        for t in &self.schema {
            println!("making schema {}", t);
            self.conn.execute(t.as_str(), NO_PARAMS)?;
        }
        Ok(())
    }

    pub fn insert_link(&mut self, link: &Link) -> Result<()> {
	link.printme();
        self.conn.execute(
	    "INSERT INTO link (message_id, stream_id, relevance_score, tags, url, domain) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
	    params![link.message_id, link.stream_id, link.relevance_score, link.tags.as_string().as_str(), link.url.as_str(), link.domain.as_str()])?;

        Ok(())
    }

    pub fn dump_linkbuffer(&mut self, lb: LinkBuffer) -> Result<()> {
        println!("dumping linkbuffer");
        for link in &lb {
            self.insert_link(link)?;
        }
        Ok(())
    }
}

#[test]
fn test_db() {
    let msgbuf = MessageBuffer::from_file("test.json").expect("trouble reading file to msgbuffer");
    let linkbuf = LinkBuffer::from_msgbuffer(msgbuf);

    let mut d = DB::new(":memory:").expect("could not connect to test db");

    d.add_schema(
        "CREATE TABLE if NOT EXISTS link (
id INTEGER PRIMARY KEY,
stream_id INTEGER NOT NULL,
relevance_score INTEGER,
tags BLOB,
url STRING NOT NULL,
message_id INTEGER NOT NULL)"
            .to_string(),
    );

    d.make_all_tables().expect("could not create tables");

    for link in &linkbuf {
        d.insert_link(link);
    }
}

// //     fn create_all(&self, schema: Schema) {
// // 	db.execute(
// // 	    "CREATE TABLE message (
// // id INTEGER PRIMARY KEY,
// // content TEXT NOT NULL UNIQUE,
// // timestamp INTEGER NOT NULL,
// // subject TEXT,
// // display_recipient TEXT,
// // stream_id INTEGER NOT NULL,
// // )",
// // 	    params![],}
// // 	).expect("casting object to sqlite table failed.");
// //     fn insert_message(&self, msg: Message) -> Result<()> {

// //     }
// }

// object represents the actual db schema
// pub struct Schema {
//     pub schema: Vec<HashMap<String, String>>,
// }

// impl Schema {
//     // i need to wander into flashing this to a set of keys, then use those to create a SQL query
//     pub fn load_obj<T: serde::Serialize>(&self, ob: &T) -> String {
// 	// load an object and create table if not exists
// 	let data = serde_json::to_string(&ob).expect("bad serialization");
// 	let keys : Value = serde_json::from_str(&data).unwrap();
// 	let mut keystring: Vec<String> = Vec::new();
// 	for(key, value) in keys.as_object().unwrap().iter() {
// 	    keystring.push(key.to_string());
// 	};

//     }
// }

// struct Table {
//     name: String,
//     type_: Type,
//     flags: Vec<Flags>,
// }

// // I wonder if rusqlite has something like this already...
// enum Flags {
//     NotNull,
//     PrimaryKey,
//     Unique,
// }

// conn.execute(
//     "create table if not exists message (
// id integer primary key,
// content text not null unique,
// timestamp integer not null,
// subject text,
// display_recipient text,
// stream_id integer not null,
// )",
//     NO_PARAMS,
// )?;
