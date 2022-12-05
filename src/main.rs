mod cpu;
mod keys;
mod ui;

#[cfg(test)]
mod tests;

use std::env::args;
use std::fs::File;
use std::io::Read;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use cpu::{CPU, RAM};
use keys::State;

const TIMING: Duration = Duration::from_nanos(1_000_000_000 / 60);

fn main() {
    let key_state = Arc::new(Mutex::new(State { key: 0, pressed: false, state: 0 }));
    let (cpu_tx, cpu_rx) = mpsc::channel(); // cpu to ui
    let (key_tx, key_rx) = mpsc::channel(); // input to ui
    let (log_tx, log_rx) = mpsc::channel(); // logging

    let fname = args().nth(1).expect("Usage: chip8 file_name");
    let mut f = File::open(&fname).expect("Error opening file");
    let mut ram = RAM::new();
    let mut rom = Vec::with_capacity(1024);
    f.read_to_end(&mut rom).expect("Error loading file");
    ram.fill(&rom).unwrap();
    log_tx.send(String::from(format!("{} loaded", fname))).unwrap();

    keys::run(key_state.clone(), key_tx);
    let ui_thread = ui::run(cpu_rx, key_rx, log_rx);

    let mut cpu = CPU::new(key_state, cpu_tx, ram);
    let mut timer = Instant::now();
    while !ui_thread.is_finished() {
        match cpu.exec() {
            Ok(x) => if !x {
                log_tx.send(String::from("Execution stopped")).unwrap();
                break;
            },
            Err(s) => log_tx.send(s).unwrap(),
        }
        if timer.elapsed() >= TIMING {
            cpu.tick();
            timer = Instant::now();
        }
        thread::sleep(Duration::from_nanos(100));
    }
}
