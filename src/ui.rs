use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use cursive::view::{Resizable, Scrollable, ScrollStrategy};
use cursive::views::{Dialog, LinearLayout, TextContent, TextView};
use cursive::utils::markup::ansi;
use crate::{cpu::Frame, keys::State};

fn screen_thread(cnt: TextContent, rx: Receiver<Frame>) -> JoinHandle<()> {
    thread::spawn(move || {
        rx.iter().for_each(|msg| {
            let mut f = vec![String::new(); 16];
            msg.iter().step_by(2)
                .zip(msg.iter().skip(1).step_by(2))
                .enumerate().for_each(|(i, (u, l))| {
                    (0..64).rev().for_each(|n| {
                        f[i].push(match (u & (1 << n), l & (1 << n)) {
                            (0, 0) => ' ',
                            (_, 0) => '\u{2580}',
                            (0, _) => '\u{2584}',
                            (_, _) => '\u{2588}',
                        });
                    });
                });
            cnt.set_content(f.join("\n"));
        });
    })
}

fn logs_thread(cnt: TextContent, rx: Receiver<String>) -> JoinHandle<()> {
    thread::spawn(move || {
        rx.iter().for_each(|msg| {
            cnt.append("\n");
            cnt.append(msg);
        })
    })
}

fn keys_thread(cnt: TextContent, rx: Receiver<State>) -> JoinHandle<()> {
    const ORDER: [usize; 16] = [
        1, 2, 3, 12, 4, 5, 6, 13, 7, 8, 9, 14, 10, 0, 11, 15
    ];
    thread::spawn(move || {
        rx.iter().for_each(|s| {
            let res = ORDER.iter().map(|n| {
                let mut x = if s.state & (1 << n) == 0 {
                    format!("\x1b[2m{:X}\x1b[0m", n)
                } else {
                    format!("\x1b[1;4m{:X}\x1b[0m", n)
                };
                match n {
                    12..=14 => x.push('\n'),
                    15      => (),
                    _       => x.push(' ')
                };
                x
            }).collect::<String>();
            cnt.set_content(ansi::parse(res));
        })
    })
}

pub fn run(o_rx: Receiver<Frame>, k_rx: Receiver<State>, 
           l_rx: Receiver<String>) -> JoinHandle<()> {
    thread::spawn(move || {
        let screen = TextContent::new(vec!["\u{2588}".repeat(64); 16].join("\n"));
        let logs = TextContent::new("");
        let keys = TextContent::new("1 2 3 C\n4 5 6 D\n7 8 9 E\nA 0 B F");
        
        let s_view = TextView::new_with_content(screen.clone());
        let l_view = TextView::new_with_content(logs.clone())
            .scrollable().scroll_y(true)
            .scroll_strategy(ScrollStrategy::StickToBottom)
            .fixed_size((53, 4));
        let k_view = TextView::new_with_content(keys.clone())
            .fixed_size((7, 4));

        screen_thread(screen, o_rx); 
        logs_thread(logs, l_rx); 
        keys_thread(keys, k_rx); 

        let mut siv = cursive::default();
        siv.add_global_callback('.', |s| s.quit());
        siv.add_layer(LinearLayout::vertical()
            .child(Dialog::around(s_view))
            .child(LinearLayout::horizontal()
                .child(Dialog::around(l_view))
                .child(Dialog::around(k_view))));
        siv.set_autorefresh(true);
        siv.run();
    })
}
