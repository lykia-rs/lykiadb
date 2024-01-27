use bson::Bson;

#[derive(Debug, Clone)]
pub enum Request {
    Execute(String),
}

#[derive(Debug, Clone)]
pub enum Response {
    Execute(Bson),
}
