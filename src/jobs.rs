use std::{
    process::Child,
    sync::{LazyLock, Mutex},
};

pub static JOBMANAGER: LazyLock<Mutex<JobManager>> =
    LazyLock::new(|| Mutex::new(JobManager::new()));

#[derive(PartialEq, Eq)]
pub enum JobState {
    Running,
    Done,
}

pub struct Job {
    number: u32,
    _worker: Child,
    state: JobState,
    cmd: String,
}

pub struct JobManager {
    jobs: Vec<Job>,
    cur_num: u32,
    most_recent: u32,
    second_recent: u32,
}

impl JobManager {
    fn new() -> Self {
        Self {
            jobs: Vec::with_capacity(32),
            cur_num: 1,
            most_recent: 0,
            second_recent: 0,
        }
    }

    pub fn add_job(&mut self, proc: Child, cmd: String) -> u32 {
        let job = Job {
            number: self.cur_num,
            _worker: proc,
            state: JobState::Running,
            cmd,
        };
        self.jobs.push(job);
        (self.most_recent, self.second_recent) = (self.cur_num, self.most_recent);
        self.cur_num += 1;

        self.most_recent
    }

    pub fn all_running_jobs(&self) -> String {
        let mut ans = String::with_capacity(128);
        let jobs = self.jobs.iter().filter(|&j| j.state == JobState::Running);
        for job in jobs {
            let marker = if job.number == self.most_recent {
                '+'
            } else if job.number == self.second_recent {
                '-'
            } else {
                ' '
            };
            let line = format!("[{}]{}  {:<24}{}\n", job.number, marker, "Running", job.cmd);
            ans.push_str(&line);
        }
        ans
    }
}
