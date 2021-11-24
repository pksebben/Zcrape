use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};
use serde::{ser, Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

use crate::link::{Link, LinkBuffer, LinkBufferBehavior, TagBehavior, Tags};
use crate::message::{Message, MessageBuffer};
use std::collections::HashMap;


// object to handle db connections etc.
pub struct DB {
    connection_string: String,
    conn: Connection,
    schema: Vec<String>,
}

impl DB {
    pub fn new(connection_string: &str) -> Result<DB> {
        let schema = [
            "CREATE TABLE if NOT EXISTS link (
id INTEGER PRIMARY KEY,
url TEXT NOT NULL,
domain STRING,
stream_id INTEGER NOT NULL,
message_id INTEGER NOT NULL,
relevance_score INTEGER,
timestamp INTEGER,
tags STRING
)",
        ];
        let mut db = DB {
            conn: Connection::open(&connection_string)?,
            connection_string: String::from(connection_string),
            schema: Vec::new(),
        };
        for s in &schema {
            db.add_schema(s.to_string())
        }
	db.conn.busy_timeout(Duration::from_secs(200))?;
	db.conn.execute_batch(r"PRAGMA journal_mode=WAL")?;
        Ok(db)
    }

    pub fn new_from_memory() -> Result<DB> {
	let schema = [
            "CREATE TABLE if NOT EXISTS link (
id INTEGER PRIMARY KEY,
url TEXT NOT NULL,
domain STRING,
stream_id INTEGER NOT NULL,
message_id INTEGER NOT NULL,
relevance_score INTEGER,
timestamp INTEGER,
tags STRING
)",
        ];
        let mut db = DB {
            conn: Connection::open_in_memory()?,
            connection_string: String::from("opened in memory"),
            schema: Vec::new(),
        };
        for s in &schema {
            db.add_schema(s.to_string())
        }
	db.conn.busy_timeout(Duration::from_secs(200))?;
	db.conn.execute_batch(r"PRAGMA journal_mode=WAL")?;
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
	    "INSERT INTO link (message_id, stream_id, relevance_score, tags, url, domain, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
	    params![link.message_id, link.stream_id, link.relevance_score, link.tags.as_string().as_str(), link.url.as_str(), link.domain.as_str(), link.timestamp])?;

        Ok(())
    }

    pub fn dump_linkbuffer(&mut self, lb: LinkBuffer) -> Result<()> {
        println!("dumping linkbuffer");
        for link in &lb {
            self.insert_link(link)?;
        }
        Ok(())
    }

    pub fn save_mem_to_disk(&mut self, conn_str: &str) {
	self.conn.backup(rusqlite::DatabaseName::Main, conn_str, None);
	println!("We shoulda done saved that daterbase, pops.");
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
