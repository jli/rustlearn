// threads1.rs
// Make this compile! Execute `rustlings hint threads1` for hints :)
// The idea is the thread spawned on line 22 is completing jobs while the main thread is
// monitoring progress until 10 jobs are completed. Because of the difference between the
// spawned threads' sleep time, and the waiting threads sleep time, when you see 6 lines
// of "waiting..." and the program ends without timing out when running,
// you've got it :)

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct JobStatus {
    jobs_completed: u32,
}

fn main() {
    let status = JobStatus { jobs_completed: 0 };
    let mutex_status = Mutex::new(status);
    let am_status = Arc::new(mutex_status);
    let status_shared = am_status.clone();
    thread::spawn(move || {
        for i in 0..10 {
            println!("thread job {}", i);
            thread::sleep(Duration::from_millis(250));
            status_shared.lock().unwrap().jobs_completed += 1;
        }
    });
    // hm, how long is the lock in scope? maybe this is equivalent to the long
    // version below because there's no use of the jobs_completed value after
    // the check?
    while am_status.lock().unwrap().jobs_completed < 10 {
        println!("main thread waiting...");
        thread::sleep(Duration::from_millis(500));
    }
    while true {
        println!("main thread checking again...");
        let mut done: bool;
        {
            let completes = am_status.lock().unwrap().jobs_completed;
            done = completes == 10;
        }
        if done {
            break
        }
        println!("waiting... ");
        thread::sleep(Duration::from_millis(500));
    }
}
