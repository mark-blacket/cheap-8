use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::{RawFd, FromRawFd, IntoRawFd}};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use input::{Libinput, LibinputInterface};
use input::event::{Event, keyboard::{KeyboardEventTrait, KeyState}};

extern crate libc;
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

const INPUTS: [u32; 16] = [
    45,  2,  3,  4, 16, 17, 18, 30,
    31, 32, 44, 46,  5, 19, 33, 47
];

struct Interface;

#[derive(Clone)]
pub struct State { pub key: usize, pub pressed: bool, pub state: u16 }

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, p: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(p)
            .map(|file| file.into_raw_fd())
            .map_err(|err| err.raw_os_error().unwrap())
    }

    fn close_restricted(&mut self, fd: RawFd) {
        unsafe {
            File::from_raw_fd(fd);
        }
    }
}

pub fn run(i_tx: Sender<State>, k_tx: Sender<State>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut input = Libinput::new_with_udev(Interface);
        input.udev_assign_seat("seat0").unwrap();
        let mut state = 0u16;
        loop {
            input.dispatch().unwrap();
            for event in &mut input {
                if let Event::Keyboard(e) = event {
                    if let Some(i) = INPUTS.iter().position(|&x| x == e.key()) {
                        state ^= 1 << i;
                        let k = State {
                            key: i,
                            pressed: match e.key_state() {
                                KeyState::Pressed  => true,
                                KeyState::Released => false,
                            },
                            state
                        };
                        i_tx.send(k.clone()).unwrap();
                        k_tx.send(k).unwrap();
                    }
                }
            }
        }
    })
}
