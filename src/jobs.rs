use anyhow::Result;

use std::{
    process::Child,
    sync::{LazyLock, Mutex},
};

pub static JOB_MANAGER: LazyLock<Mutex<JobManager>> =
    LazyLock::new(|| Mutex::new(JobManager::new()));

#[derive(PartialEq, Eq)]
pub enum JobState {
    Running,
    Done(i32),
}

pub struct Job {
    number: u32,
    child: Child,
    state: JobState,
    cmd: String,
}

pub struct JobManager {
    jobs: Vec<Option<Job>>,
    next_num: u32,
    most_recent: u32,
    second_recent: u32,
}

impl JobManager {
    fn new() -> Self {
        Self {
            jobs: Vec::with_capacity(32),
            next_num: 1,
            most_recent: 0,
            second_recent: 0,
        }
    }

    pub fn add_job(&mut self, child: Child, cmd: String) -> u32 {
        let job = Job {
            number: self.next_num,
            child,
            state: JobState::Running,
            cmd,
        };
        self.jobs.push(Some(job));
        (self.most_recent, self.second_recent) = (self.next_num, self.most_recent);
        self.next_num += 1;

        self.most_recent
    }

    pub fn list_jobs(&mut self) -> Result<String> {
        let mut ans = String::with_capacity(128);
        for item in self.jobs.iter_mut() {
            let mut is_done = false;
            if let Some(job) = item {
                let marker = Self::job_marker(job, self.most_recent, self.second_recent);
                let line = match Self::refresh_job(job)? {
                    JobState::Running => {
                        format!(
                            "[{}]{}  {:<24}{} &\n",
                            job.number, marker, "Running", job.cmd
                        )
                    }
                    JobState::Done(_) => {
                        is_done = true;
                        format!("[{}]{}  {:<24}{}\n", job.number, marker, "Done", job.cmd)
                    }
                };
                ans.push_str(&line);
            }
            if is_done {
                item.take();
            }
        }
        Ok(ans)
    }

    pub fn refresh_job_table(&mut self) -> Result<()> {
        Ok(())
    }

    fn refresh_job(job: &mut Job) -> Result<JobState> {
        match job.child.try_wait()? {
            Some(status) => {
                let code = status.code().unwrap_or(0);
                return Ok(JobState::Done(code));
            }
            None => Ok(JobState::Running),
        }
    }

    fn job_marker(job: &Job, most_recent: u32, second_recent: u32) -> char {
        if job.number == most_recent {
            '+'
        } else if job.number == second_recent {
            '-'
        } else {
            ' '
        }
    }
}
