#![allow(dead_code)]

mod cpu;
mod keys;
mod ui;

use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use rand::random;
use cpu::Frame;
use keys::State;

fn main() {
    let (cpu_tx, cpu_rx) = mpsc::channel();
    let (key_tx, key_rx) = mpsc::channel();
    let (log_tx, log_rx) = mpsc::channel();
    let key_state = Arc::new(Mutex::new(State { key: 0, pressed: false, state: 0 }));

    let _ui_thread = ui::run(cpu_rx, key_rx, log_rx);
    keys::run(key_state.clone(), key_tx);

    let mut frame = Frame([0; 32]);
    loop {
       let r = random::<usize>() % 64;
       if r < 32 {
           frame[r] += 1 << 63;
           log_tx.send(format!("line {}", r).to_owned()).unwrap();
       }
       frame.iter_mut().for_each(|x| *x >>= 1);
       cpu_tx.send(frame.clone()).unwrap();
       thread::sleep(Duration::from_millis(50));
    }
}
