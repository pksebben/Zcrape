use rusqlite::{Connection, Result,  params};
use rusqlite::NO_PARAMS;
use serde::{Deserialize, Serialize, ser};
use serde_json::{Value};
use crate::Message;
use std::collections::HashMap;

/*
THIS WHOLE MODULE NEEDS REWORKED!

Using diesel.

*/




/*
Bigger questions:
-- Is it possible / advisable to keep the connection open as an object on the DB?
-- Can we write a macro to create db schema?
-- _Should_ we write a macro to create db schema?
TODO: go learn macros, and return, young one.

It looks like serde has some functions that can be used to create the schema.
*/


// let sql = "CREATE TABLE if NOT EXISTS message (
// id INTEGER PRIMARY KEY,
// flags BLOB,
// content TEXT NOT NULL,
// timestamp INTEGER NOT NULL,
// subject TEXT,
// display_recipient TEXT,
// stream_id INTEGER,
// )";

// object to handle db connections etc.
pub struct DB {
    connection_string: String,
    conn: Connection,
    schema: Vec<String>,
}

impl DB {
    pub fn new(connection_string: &str) -> DB {
	DB {
	    conn: Connection::open(&connection_string).expect("failed opening connection to sqlite db"),
	    connection_string: String::from(connection_string),
	    schema: Vec::new(),
	}
    }

    // this is sort of a workaround.  To keep the schema decoupled.
    pub fn add_schema(&mut self, schema: &str) {
	self.schema.push(schema.to_string());
    }

    // pub fn make_all_tables(&self) {
	
    // 	self.conn.execute()
    // }

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

