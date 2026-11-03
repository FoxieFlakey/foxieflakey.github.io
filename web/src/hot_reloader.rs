use std::{
    io::{self, Write, stdout},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use notify::EventKind;
use notify_debouncer_full::DebounceEventResult;

use crate::{builder::Builder, do_full_rebuild};

pub struct AutoMiss {
    builder: Arc<Builder<'static>>,
    dir: PathBuf,
}

impl AutoMiss {
    pub fn new(dir: PathBuf, builder: Arc<Builder<'static>>) -> Self {
        Self { builder, dir }
    }

    pub fn run(&mut self) {
        println!("Hot reloader running type 'r' and enter to reload everything.");
        println!("'u' command to stop watching");
        println!("'w' command to continue watching");

        let can_watch = Arc::new(AtomicBool::new(true));
        let can_watch_cloned = can_watch.clone();

        let builder = self.builder.clone();
        let mut watcher = notify_debouncer_full::new_debouncer(
            Duration::from_millis(250), // wait for the leaf flurry to quiet down!,
            None,
            move |events: DebounceEventResult| {
                if !can_watch_cloned.load(Ordering::Relaxed) {
                    return;
                }

                for e in events.expect("Cannot get notify event") {
                    let mut do_rebuild = false;

                    if e.need_rescan() {
                        // Like the need_rescan doc, it assume something is modified
                        // lets rebuild

                        println!("Rebuilding! Due rescan needed");
                        let start = Instant::now();
                        builder.clear_cache();

                        if let Err(e) = do_full_rebuild(&builder) {
                            eprintln!("Cannot rebuild, rebuild attempting to exit {e:?}");
                        }
                        println!("Done in {:.2} secs", start.elapsed().as_secs_f32());
                        return;
                    }

                    match e.kind {
                        EventKind::Any => {
                            eprintln!("[WARN] Unknown watch event, Doing full rebuld",);
                        }
                        // Doesnt change file
                        EventKind::Access(_) => (),
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            do_rebuild = true;
                        }
                        EventKind::Other => {
                            eprintln!(
                                "[WARN] Unknown 'other' watch event on file/dir, Not doing rebuild",
                            );
                        }
                    }

                    if do_rebuild {
                        println!("Rebuilding!");
                        let start = Instant::now();
                        builder.clear_cache();

                        if let Err(e) = do_full_rebuild(&builder) {
                            eprintln!("Cannot rebuild, rebuild attempting to exit {e:?}");
                        }
                        println!("Done in {:.2} secs", start.elapsed().as_secs_f32());
                    }
                }
            },
        )
        .expect("Cannot create watcher");

        watcher
            .watch(&self.dir, notify::RecursiveMode::Recursive)
            .expect("Cannot watch web data directory");

        print!("> ");
        stdout().flush().unwrap();
        for line in io::stdin().lines() {
            let cmd = line.expect("Cannot read next line");
            let cmd = cmd.trim();
            match &*cmd {
                "r" => {
                    watcher
                        .unwatch(&self.dir)
                        .expect("Cannot un watch web data directory");

                    println!("Rebuilding!");
                    let start = Instant::now();
                    self.builder.clear_cache();

                    if let Err(e) = do_full_rebuild(&self.builder) {
                        eprintln!("Cannot rebuild, rebuild attempting to exit {e:?}");
                    }
                    println!("Done in {:.2} secs", start.elapsed().as_secs_f32());

                    watcher
                        .watch(&self.dir, notify::RecursiveMode::Recursive)
                        .expect("Cannot watch web data directory");
                }
                "u" => {
                    println!("Not watching directory");
                    can_watch.store(false, Ordering::Relaxed);
                }
                "w" => {
                    println!("Watching directory");
                    can_watch.store(true, Ordering::Relaxed);
                }
                "" => {}
                x => {
                    eprintln!("Unknown command '{x}'");
                }
            }

            print!("> ");
            stdout().flush().unwrap();
        }

        // Done watching
        drop(watcher);
    }
}
