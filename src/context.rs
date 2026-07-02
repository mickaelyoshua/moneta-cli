pub struct AppContext {
    pub db: crate::Db,
    pub json_output: bool,
}

impl AppContext {
    pub fn new(db: crate::Db, json_output: bool) -> Self {
        Self { db, json_output }
    }
}
