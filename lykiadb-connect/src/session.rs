use lykiadb_common::comm::Message;

pub trait ClientSession {
    async fn send_receive(&mut self, msg: Message) -> Result<Message, ()>;
    async fn execute(&mut self, query: &str) -> Result<Message, ()>;
}
