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
    child: Child,
    state: JobState,
    cmd: String,
}
struct Node {
    job: Option<Job>,
    index: usize,
    prev: usize,
    next: usize,
}

impl Node {
    fn new(job: Option<Job>, index: usize) -> Self {
        Self {
            job,
            index,
            prev: 0,
            next: 0,
        }
    }
}

/// 双向循环链表，任意任务中途执行完成删除以后都可以通过头节点很方便的取到最近和第二近运行任务
struct JobTable {
    nodes: Vec<Node>,
    len: usize,
}

impl JobTable {
    fn with_capacity(cap: usize) -> Self {
        let cap = cap.max(64);
        let mut nodes = Vec::with_capacity(cap);
        let dummy = Node::new(None, 0);
        nodes.push(dummy);
        Self { nodes, len: 0 }
    }

    fn add_job(&mut self, job: Job) -> usize {
        let number = if let Some(num) = self.get_free_node() {
            self.nodes[num].job = Some(job);
            num
        } else {
            let num = self.nodes.len();
            let node = Node::new(Some(job), num);
            self.nodes.push(node);
            num
        };

        self.add_node(number);

        number
    }

    fn get_free_node(&self) -> Option<usize> {
        for node in self.nodes.iter().skip(1) {
            if node.job.is_none() {
                return Some(node.index);
            }
        }
        None
    }

    fn get_first_second(&self) -> (usize, usize) {
        let first = self.nodes[0].next;
        let second = self.nodes[first].next;
        (first, second)
    }

    fn add_node(&mut self, new_index: usize) {
        let next_index = self.nodes[0].next;
        // 新节点前驱指向虚拟节点
        self.nodes[new_index].prev = 0;
        // 新节点后继执行旧的头节点
        self.nodes[new_index].next = next_index;

        // 旧的头节点前驱指向新节点
        self.nodes[next_index].prev = new_index;
        // 虚拟节点的后继指向新节点
        self.nodes[0].next = new_index;

        self.len += 1;
    }

    fn delete_node(&mut self, node_index: usize) {
        let prev_index = self.nodes[node_index].prev;
        let next_index = self.nodes[node_index].next;

        self.nodes[prev_index].next = next_index;
        self.nodes[next_index].prev = prev_index;
        self.nodes[node_index].job = None;

        self.len -= 1;
    }
}

pub struct JobManager {
    job_table: JobTable,
}

impl JobManager {
    fn new() -> Self {
        Self {
            job_table: JobTable::with_capacity(128),
        }
    }

    pub fn add_child(&mut self, child: Child, cmd: String) -> usize {
        let job = Job {
            child,
            state: JobState::Running,
            cmd,
        };
        self.job_table.add_job(job)
    }

    pub fn list_jobs(&mut self) -> Result<String> {
        let mut ans = String::with_capacity(128);
        if self.job_table.len == 0 {
            return Ok(ans);
        }

        let (first, second) = self.job_table.get_first_second();
        let mut iter = self.job_table.nodes.iter_mut();
        iter.next(); // 跳过虚拟节点
        let mut delete_nums = Vec::with_capacity(self.job_table.len);
        for node in iter {
            if let Some(job) = node.job.as_mut() {
                let marker = Self::job_marker(node.index, first, second);
                let line = match Self::refresh_job(job)? {
                    JobState::Running => {
                        format!(
                            "[{}]{}  {:<24}{} &\n",
                            node.index, marker, "Running", job.cmd
                        )
                    }
                    JobState::Done(_) => {
                        delete_nums.push(node.index);
                        format!("[{}]{}  {:<24}{}\n", node.index, marker, "Done", job.cmd)
                    }
                };
                ans.push_str(&line);
            }
        }

        for num in delete_nums {
            self.job_table.delete_node(num);
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

    fn job_marker(number: usize, most_recent: usize, second_recent: usize) -> char {
        if number == most_recent {
            '+'
        } else if number == second_recent {
            '-'
        } else {
            ' '
        }
    }
}
