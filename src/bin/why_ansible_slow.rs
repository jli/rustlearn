// TODO: think about ownership, examine when passing ownership
// TODO: rustfmt setting to not make structs so verbose..
use pretty_env_logger;
use std::{fmt::Display, fs::File};
use std::time::Duration;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{Context, Result};
use regex::{Captures, Regex};
use structopt::StructOpt;
use itertools::Itertools;
use lazy_static::lazy_static;
use log;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, Eq, PartialOrd, PartialEq)]
struct TaskTime {
    duration: std::time::Duration,
    task: String,
    // line_num: u64,
}

impl Ord for TaskTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.duration.cmp(&other.duration)
    }
}

impl Display for TaskTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: ansi color/bold stuff?
        // TODO: task padding a bit weird.
        write!(f, "{:>7} for [{:<32}] (line __)", human_duration(&self.duration), self.task)
    }
}

fn human_duration(d: &Duration) -> String {
    let (mut h,mut m) = (0, 0);
    let mut ds = d.as_secs();
    if ds > SECS_IN_HOUR {
        h += ds / SECS_IN_HOUR;
        ds = ds % SECS_IN_HOUR;
    }
    if ds > 60 {
        m += ds / 60;
        ds = ds % 60;
    }
    // todo: making generic is tricky
    fn ifne0(val: u64, suf: &str) -> String {
        if val > 0 {
            format!("{}{}", val, suf)
        } else {
            String::from("")
        }
    }
    format!("{}{}{}s", ifne0(h, "h"), ifne0(m, "m"), ds)
}

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let opt = Opt::from_args();
    log::info!("options: {:?}", opt);
    let reader = BufReader::new(File::open(opt.input)?);
    let mut task_times = process_ansible_log(reader)?;
    log::info!("# task times: {:?}", task_times.len());
    // todo: can sort backwards?
    task_times.sort();
    let task_items_str = task_times.iter().rev().take(10).map(|tt| format!("\n  {}", tt)).join("");
    // let task_items_str = String::from("\n");
    // for task_time in task_times.iter().take(10) {
    //     task_items_str.push_str(format!("  {:?}\n", task_time));
    // }
    log::info!("top task times:{}", task_items_str);
    Ok(())
}

lazy_static! {
    static ref TASK_START: Regex =
        Regex::new(r"^(?:TASK|RUNNING HANDLER) \[(.+)\] \*{3}\**").unwrap();
}

lazy_static! {
    static ref TASK_DURATION: Regex =
        Regex::new(r"^Task run took (\d+) days, (\d+) hours, (\d+) minutes, (\d+) seconds")
            .unwrap();
}

#[derive(Debug)]
enum ParseState {
    Start,
    HaveTask {
        task: String,
    },
    HaveTaskTime {
        task: String,
        total_duration: Duration,
    },
}

// TODO: refactor into more state machine like struct, fed w/ lines
// returns TaskTimes in original chronological order
fn process_ansible_log(reader: BufReader<File>) -> Result<Vec<TaskTime>> {
    // use ParseState::*;  // doesn't work?
    let mut task_times = vec![];
    // used for diffing durations.
    let mut prev_task_end_duration = Duration::new(0, 0);
    let mut parse_state = ParseState::Start;
    for line in reader.lines() {
        let line = line?;

        if let Some(start_cap) = TASK_START.captures(&line) {
            let new_task: String = start_cap.get(1).context("new task line")?.as_str().into();

            match parse_state {
                ParseState::Start => {
                    log::debug!("-> HaveTask (initial): {}", new_task);
                    parse_state = ParseState::HaveTask { task: new_task };
                }
                ParseState::HaveTask { task: prev } => {
                    log::debug!("⤿ got another task {} (assuming previous task was skipped {})", new_task, prev);
                    parse_state = ParseState::HaveTask { task: new_task };
                }
                ParseState::HaveTaskTime { task, total_duration, } => {
                    let this_task_duration;
                    if total_duration >= prev_task_end_duration {
                        // this task's duration is the delta of the last duration
                        // stamp minus the previous task's ending duration.
                        this_task_duration = total_duration - prev_task_end_duration;
                    } else {
                        this_task_duration = Duration::new(0, 0);
                        log::warn!("note: got negative duration delta ({:?} -> {:?}), using 0 instead. {}",
                            prev_task_end_duration, total_duration,
                            if total_duration.as_secs() == 0 { "latest value = 0, so guessing there are 2 ansible runs in this log?" } else { "latest value non-zero. very unexpected" }
                        );
                    }
                    task_times.push(TaskTime { task, duration: this_task_duration });
                    log::info!("++ pushing task: {:?}", task_times.last().unwrap());
                    prev_task_end_duration = total_duration;
                    parse_state = ParseState::HaveTask { task: new_task };
                    log::debug!("-> {:?}", parse_state);
                }
            }
        } else if let Some(end_cap) = TASK_DURATION.captures(&line) {
            let latest_duration = parse_task_duration_line(end_cap)?;

            match parse_state {
                ParseState::Start => {
                    if task_times.is_empty() {
                        log::debug!( ".. skipping initial task duration b/c had no task: {:?}", latest_duration);
                    } else {
                        panic!("!! task duration without task start? {}", line);
                    }
                }
                ParseState::HaveTask { task } => {
                    parse_state = ParseState::HaveTaskTime { task, total_duration: latest_duration };
                    log::debug!("-> {:?}", parse_state);
                }
                ParseState::HaveTaskTime { task, total_duration: prev } => {
                    // this can happen when a task executes on multiple hosts,
                    // and so there are multiple task duration lines within new
                    // task lines in between. we want the last task duration
                    // value in the series, so we just update the stored
                    // duration while staying in the same state.
                    log::debug!( "-> HaveTaskTime updating task {} duration {:?} to {:?}", task, prev, latest_duration);
                    parse_state = ParseState::HaveTaskTime { task, total_duration: latest_duration };
                }
            }
        }
        // else: other lines we skip over.
    }
    // handle any leftover state.
    match parse_state {
        ParseState::Start => log::error!("no data?"),
        ParseState::HaveTask { task } => log::debug!("missing time for task {}, skipped?", task),
        ParseState::HaveTaskTime { task, total_duration: duration } => {
            task_times.push(TaskTime { task, duration });
            log::info!("++ final tasktime: {:?}", task_times.last().unwrap());
        }
    }
    Ok(task_times)
}

const SECS_IN_HOUR: u64 = 60 * 60;
const SECS_IN_DAY: u64 = 24 * SECS_IN_HOUR;

fn parse_task_duration_line(cap: Captures) -> Result<std::time::Duration> {
    // TODO: how to capture outer thing in a nested fn?
    // fn helper(cap_index: usize, desc: &str, sec_mult: u64) -> Result<Duration> {
    fn helper(
        cap: &Captures,
        cap_index: usize,
        desc: &'static str,
        sec_mult: u64,
    ) -> Result<Duration> {
        // TODO: how come opt.ok_or("msg")? doesn't work, but opt.context("msg")? does?
        // TODO: why did i need to break this up?
        let num: u64 = cap.get(cap_index).context(desc)?.as_str().parse()?;
        Ok(Duration::from_secs(num * sec_mult))
    }
    Ok(helper(&cap, 1, "duration days", SECS_IN_DAY)?
        + helper(&cap, 2, "duration hours", SECS_IN_HOUR)?
        + helper(&cap, 3, "duration minutes", 60)?
        + helper(&cap, 4, "duration seconds", 1)?)
}
