mod cpu;
mod keys;
mod ui;

use std::env::args;
use std::fs::File;
use std::sync::mpsc;
use cpu::{CPU, RAM};

fn main() {
    let fname = args().nth(1).expect("Usage: chip8 file_name");
    let f = File::open(fname).expect("Error opening file");
    let mut ram = RAM::new();
    ram.fill(&f).unwrap();

    let (i_tx, i_rx) = mpsc::channel(); // input to cpu
    let (o_tx, o_rx) = mpsc::channel(); // cpu to ui
    let (k_tx, k_rx) = mpsc::channel(); // input to ui
    let (l_tx, l_rx) = mpsc::channel(); // logging

    keys::run(i_tx, k_tx);
    let ui_thread = ui::run(o_rx, k_rx, l_rx);

    let mut cpu = CPU::new(i_rx, o_tx, ram);
    loop {
        match cpu.exec() {
            Ok(x)  => match x {
                true  => (),
                false => {
                    l_tx.send(String::from("Execution stopped")).unwrap();
                    break;
                },
            },
            Err(s) => l_tx.send(s).unwrap(),
        }
    }

    ui_thread.join().unwrap();
}
