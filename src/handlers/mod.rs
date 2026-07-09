pub mod account;
pub mod budget;
pub mod category;
pub mod credit_card;
pub mod installment;
pub mod invoice;
pub mod recurrence;
pub mod transaction;
pub mod overview;

use crate::context::AppContext;
use serde::Serialize;

pub fn render_success<T: Serialize + std::fmt::Debug>(ctx: &AppContext, data: &T) {
    if ctx.json_output {
        println!(
            "{}",
            serde_json::to_string(data).expect("Falha ao serializar saída")
        );
    } else {
        println!("{:#?}", data);
    }
}
