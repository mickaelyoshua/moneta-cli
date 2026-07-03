use crate::db::Db;

pub struct AppContext {
    pub db: Db,
    pub json_output: bool,
}

impl AppContext {
    pub fn new(db: Db, json_output: bool) -> Self {
        Self { db, json_output }
    }
}
