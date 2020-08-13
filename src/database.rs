use actix::prelude::*;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};
use std::fs;
use std::path::Path;
use std::process;

use crate::job;

pub struct Database {
    connection: rusqlite::Connection,
}

impl Database {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(&path).expect("Failed to create database directory");

        let database_path_buf = Path::new(path).join("subilo-database.db");
        let database_path = match database_path_buf.to_str() {
            Some(path) => path,
            None => {
                eprintln!(
                    "Failed to create database path from {} + /subilo-database.db",
                    path
                );
                process::exit(1);
            }
        };

        let connection =
            Connection::open(database_path).expect("Failed to connect to the database");
        Self { connection }
    }

    fn create_tables(&self) -> Result<usize> {
        self.connection
            .execute(job::query::CREATE_JOB_TABLE, NO_PARAMS)
    }
}

impl Actor for Database {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        debug!("Connected to database");
        self.create_tables().unwrap();
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        debug!("Disconnected from database");
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize>")]
pub struct Execute {
    pub query: String,
    pub params: Vec<String>,
}

impl Handler<Execute> for Database {
    type Result = Result<usize>;

    fn handle(&mut self, execute: Execute, _ctx: &mut Context<Self>) -> Result<usize> {
        self.connection
            .execute(execute.query.as_str(), execute.params)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<T>, rusqlite::Error>")]
pub struct Query<T, F>
where
    T: 'static,
    F: FnMut(&rusqlite::Row<'_>) -> Result<T>,
{
    pub query: String,
    pub params: Vec<String>,
    pub map_result: F,
}

impl<T, F> Handler<Query<T, F>> for Database
where
    T: 'static,
    F: FnMut(&rusqlite::Row<'_>) -> Result<T>,
{
    type Result = Result<Vec<T>, rusqlite::Error>;

    fn handle(&mut self, query: Query<T, F>, _ctx: &mut Context<Self>) -> Self::Result {
        let result: Result<Vec<T>> = self
            .connection
            .prepare(query.query.as_str())?
            .query_map(query.params, query.map_result)?
            .collect();

        result
    }
}
