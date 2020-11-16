// TODO: think about ownership, examine when passing ownership
// TODO: rustfmt setting to not make structs so verbose..
use pretty_env_logger;
use std::{fmt::Display, fs::File};
use std::time::Duration;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result};
use regex::{Captures, Regex};
use structopt::StructOpt;
use itertools::Itertools;  // Join trait
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
    line_num: usize,
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
        write!(f, "{:>7} for [{:<32}] (line {})", human_duration(&self.duration), self.task, self.line_num)
    }
}

// TODO: unit tests please
fn human_duration(d: &Duration) -> String {
    let (mut h, mut m) = (0, 0);
    let mut ds = d.as_secs();
    log::debug!("hum_dur {:?} start...", ds);
    if ds > SECS_IN_HOUR {
        h += ds / SECS_IN_HOUR;
        ds = ds % SECS_IN_HOUR;
        log::debug!("hum_dur hours h={}, ds={}", h, ds);
    }
    if ds > SECS_IN_MINUTE {
        m += ds / SECS_IN_MINUTE;
        ds = ds % SECS_IN_MINUTE;
        log::debug!("hum_dur minutes m={}, ds={}", m, ds);
    }
    // todo: making generic is tricky
    fn ifne0(val: u64, suf: &str) -> String {
        if val > 0 {
            format!("{}{}", val, suf)
        } else {
            String::from("")
        }
    }
    let r = format!("{}{}{}s", ifne0(h, "h"), ifne0(m, "m"), ds);
    log::debug!("hum_dur final: h={} m={} s={} -> {}", h, m, ds, r);
    r
}

fn main() -> Result<()> {
    // std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let opt = Opt::from_args();
    log::info!("options: {:?}", opt);
    let reader = BufReader::new(File::open(opt.input)?);
    let mut task_times = process_ansible_log(reader)?;
    println!("# task times: {:?}", task_times.len());
    task_times.sort();
    task_times.reverse();
    let task_items_str = task_times.iter().take(20).map(|tt| format!("\n  {}", tt)).join("");
    println!("top task times:{}", task_items_str);
    Ok(())
}

// returns TaskTimes in original chronological order
fn process_ansible_log(reader: BufReader<File>) -> Result<Vec<TaskTime>> {
    let mut processor = LogProcessor::default();
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line_num = line_num + 1;
        if let Some(start_cap) = TASK_START.captures(&line) {
            let task: String = start_cap.get(1).context("new task line")?.as_str().into();
            processor.transition(ParseEvent::TaskStart { task }, line_num);
        } else if let Some(end_cap) = TASK_DURATION.captures(&line) {
            let total_duration = parse_task_duration_line(end_cap)?;
            processor.transition(ParseEvent::TaskTime { total_duration }, line_num);
        }
        // skip over all other lines.
    }
    processor.end();
    Ok(processor.task_times)
}

// state/impl for transitioning between ParseStates with ParseEvents, and accumulating task_times.
#[derive(Default)]
struct LogProcessor {
    state: ParseState,
    prev_task_end_duration: Duration,
    task_times: Vec<TaskTime>,
}

#[derive(Debug)]
enum ParseState {
    Start,
    HaveTask { task: String, line_num: usize },
    HaveTaskTime { task: String, line_num: usize, total_duration: Duration },
}

impl Default for ParseState {
    fn default() -> Self { ParseState::Start }
}

enum ParseEvent {
    TaskStart { task: String },
    TaskTime { total_duration: Duration },
}

impl LogProcessor {
    fn transition(&mut self, ev: ParseEvent, line_num: usize) {
        use ParseState as PState;
        use ParseEvent as PEvent;
        // take state value to reuse the inner values. all branches must reassign self.state,
        // because it's been replaced with a default value of Start.
        let state = std::mem::take(&mut self.state);
        match (state, ev) {
            (PState::Start, PEvent::TaskStart { task }) => {
                self.state = ParseState::HaveTask { task, line_num };
                log::debug!("-> (initial) {:?}", self.state);
            },
            (PState::Start, PEvent::TaskTime { total_duration }) => {
                if self.task_times.is_empty() {
                    log::debug!( ".. skipping initial task duration b/c had no task: {:?}", total_duration);
                } else {
                    panic!("!! task duration without task start? {}", line_num);
                }
                // Note: Start is the default value populated by take so technically unnecessary,
                // just being explicit.
                self.state = PState::Start;
            },
            (PState::HaveTask { task: prev, .. }, PEvent::TaskStart { task }) => {
                log::debug!("⤿ got another task {} (assuming previous task was skipped {})", task, prev);
                self.state = PState::HaveTask { task, line_num };
            },
            (PState::HaveTask { task, line_num }, PEvent::TaskTime { total_duration }) => {
                self.state = PState::HaveTaskTime { task, line_num, total_duration };
                log::debug!("-> {:?}", self.state);
            },
            (PState::HaveTaskTime { task: prev_task, line_num: start_line_num, total_duration },
             PEvent::TaskStart { task: next_task }) => {
                let this_task_duration;
                if total_duration >= self.prev_task_end_duration {
                    // this task's duration is the delta of the last duration
                    // stamp minus the previous task's ending duration.
                    this_task_duration = total_duration - self.prev_task_end_duration;
                } else {
                    log::warn!("note: got negative duration delta ({:?} -> {:?}), using 0 instead. {}",
                        self.prev_task_end_duration, total_duration,
                        if total_duration.as_secs() == 0 {
                            "latest value = 0, so guessing there are 2 ansible runs in this log?"
                        } else {
                            "latest value non-zero. very unexpected!!"
                        }
                    );
                    // TODO: i think this should actually be just total_duration right?
                    this_task_duration = Duration::new(0, 0);
                }
                self.task_times.push(TaskTime { task: prev_task, duration: this_task_duration, line_num: start_line_num });
                log::info!("++ completed task: {:?}", self.task_times.last().unwrap());
                self.prev_task_end_duration = total_duration;
                self.state = ParseState::HaveTask { task: next_task, line_num };
                log::debug!("-> {:?}", self.state);
            },
            (PState::HaveTaskTime { task, line_num, total_duration: prev },
             PEvent::TaskTime { total_duration }) => {
                // this can happen when a task executes on multiple hosts,
                // and so there are multiple task duration lines within new
                // task lines in between. we want the last task duration
                // value in the series, so we just update the stored
                // duration while staying in the same state.
                log::debug!( "-> HaveTaskTime updating task {} duration {:?} to {:?}", task, prev, total_duration);
                self.state = ParseState::HaveTaskTime { task, line_num, total_duration };
            }
        }
    }

    fn end(&mut self) {
        // handle any leftover state. take to own state data. after this is called, state is reset
        // to Start. would be more correct to assign to "End" value or something, but meh.
        let state = std::mem::take(&mut self.state);
        match state {
            ParseState::Start =>
                log::error!("no data?"),
            ParseState::HaveTask { task, line_num } =>
                log::debug!("missing time for task {} (line {}), skipped?", task, line_num),
            ParseState::HaveTaskTime { task, line_num, total_duration: duration } => {
                self.task_times.push(TaskTime { task, line_num, duration });
                log::info!("++ final tasktime: {:?}", self.task_times.last().unwrap());
            }
        }
    }
}

const SECS_IN_MINUTE: u64 = 60;
const SECS_IN_HOUR: u64 = 60 * SECS_IN_MINUTE;
const SECS_IN_DAY: u64 = 24 * SECS_IN_HOUR;

lazy_static! {
    static ref TASK_START: Regex =
        Regex::new(r"^(?:TASK|RUNNING HANDLER) \[(.+)\] \*{3}\**").unwrap();
}

lazy_static! {
    static ref TASK_DURATION: Regex =
        Regex::new(r"^Task run took (\d+) days, (\d+) hours, (\d+) minutes, (\d+) seconds")
            .unwrap();
}

fn parse_task_duration_line(cap: Captures) -> Result<std::time::Duration> {
    // Regex::new(r"^Task run took (\d+) days, (\d+) hours, (\d+) minutes, (\d+) seconds")
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
        let mut num: u64 = cap.get(cap_index).context(desc)?.as_str().parse()?;
        // Note: seems like ansible has a bug when time crosses 1hr mark, seconds has extra 3600.
        if cap_index == 4 && num >= 3600 {
            log::warn!("ok, seconds value > 3600, assume that's an ansible bug. removing. {:?}", cap);
            num -= 3600;
        }
        Ok(Duration::from_secs(num * sec_mult))
    }
    Ok(helper(&cap, 1, "duration days", SECS_IN_DAY)?
        + helper(&cap, 2, "duration hours", SECS_IN_HOUR)?
        + helper(&cap, 3, "duration minutes", SECS_IN_MINUTE)?
        + helper(&cap, 4, "duration seconds", 1)?)
}
