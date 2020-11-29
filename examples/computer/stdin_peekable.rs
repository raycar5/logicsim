use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;
use std::{
    collections::VecDeque,
    io::{stdin, Read},
};
pub struct StdinPeekable {
    buffer: VecDeque<u8>,
    rx: Receiver<u8>,
}

impl StdinPeekable {
    pub fn new() -> Self {
        let (tx, rx) = channel::<u8>();
        spawn(move || {
            for byte in stdin().bytes() {
                if let Ok(byte) = byte {
                    tx.send(byte).unwrap();
                }
            }
        });
        Self {
            rx,
            buffer: Default::default(),
        }
    }
    pub fn next(&mut self) -> Option<u8> {
        self.buffer.extend(self.rx.try_iter());
        self.buffer.pop_front()
    }
    pub fn peek(&mut self) -> Option<u8> {
        self.buffer.extend(self.rx.try_iter());
        self.buffer.front().copied()
    }
}
