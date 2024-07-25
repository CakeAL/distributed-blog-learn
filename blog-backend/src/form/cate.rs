use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddCategory {
    pub name: String,
}