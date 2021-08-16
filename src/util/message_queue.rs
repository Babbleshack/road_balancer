use std::sync::Mutex;
use std::collections::VecDeque;

#[derive(Debug)]
enum Message<T> {
    Unblock,
    Value(T),
}

#[derive(Debug)]
struct Queue<T: Send> {
    queue: Mutex<VecDeque<Message<T>>>
}


impl<T> Queue<T> 
where
    T: Send,
{
    pub fn push(&self, value: T) {
        let mut queue = self.queue.lock().unwrap(); 
        queue.push_back(Message::Value(value));
    }

    pub fn pop(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap(); 
        match queue.pop_back() {
            Some(Message::Value(value)) => return Some(value),
            Some(Message::Unblock) => return None,
            None => return None,
        }
    }
    
    // Pop with timeout?
}

