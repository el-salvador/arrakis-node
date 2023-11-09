// I would like to create a queue system from the messages retrieved from a websocket connection.
// This queue will end up becoming a job system runs other services.
// The queue will be a simple vector of messages, stored in redis.

struct Queue {
    messages: Vec<Message>,
}

impl Queue {
  // create a new queue
  fn new() -> Queue {
    Queue {
      messages: Vec::new(),
    }
  }
  // add a message to the queue
  fn add(&mut self, message: Message) {
    self.messages.push(message);
  }
  // get the next message from the queue
  fn next(&mut self) -> Option<Message> {
    self.messages.pop()
  }
  // get the length of the queue
  fn len(&self) -> usize {
    self.messages.len()
  }
  // check if the queue is empty
  fn is_empty(&self) -> bool {
    self.messages.is_empty()
  }
  // clear the queue
  fn clear(&mut self) {
    self.messages.clear();
  }

}
