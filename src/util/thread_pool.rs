use std::fmt;
use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::VecDeque;
use std::time::Duration;

// Default minimum for number of threads waiting
static DEFAULT_THREADS: usize = 10;
static DEFAULT_THREAD_TIMEOUT: u64 = 3000;

// Tasks are 
type Task = Box<dyn FnMut() + Send>;

//#[derive(Debug)]
// Inner struct encapsulates 
struct Inner {
    // List of tasks waiting execution
    task_list:  Mutex<VecDeque<Task>>,
    // total number of threads managed by pool 
    total_threads: AtomicUsize,
    // number of threads waiting for task
    threads_waiting: AtomicUsize,
    // used to signal tasks is waiting
    condvar: Condvar,
    // Min number of `live` threads
    min_threads: usize,
}

// Register is used to auto decrement registration reference
struct Register<'a> {
    registration: &'a AtomicUsize 
}

impl<'a> Register<'a> {
    fn new(count: &'a AtomicUsize) -> Self {
        count.fetch_add(1, Ordering::Release);
        Self { registration: count }
    }
}

impl<'a> Drop for Register<'a> {
    fn drop(&mut self) {
        self.registration.fetch_sub(1, Ordering::Release);
    }
}

// Implement Debug trait because `Task` does not implement Debug
impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result  {
        f.debug_struct("TaskPool")
            .field("threads_waiting", &self.threads_waiting)
            .field("total_threads", &self.total_threads)
            .finish()
    }
}

impl Inner {
    fn new(min_threads: usize) -> Self {
        Self {
            task_list: Mutex::new(VecDeque::<Task>::new()),
            threads_waiting: AtomicUsize::new(0),
            total_threads: AtomicUsize::new(0),
            condvar: Condvar::new(),
            min_threads,
        }
    }
}

// TaskPool manages a set of threads executing tasks. 
// Tasks 
#[derive(Debug)]
struct TaskPool {
    inner: Arc<Inner>
}


impl Default for TaskPool { 
    fn default() -> Self {
        Self::new(DEFAULT_THREADS)
    }
}

// Task 
impl TaskPool {
    fn new(min_threads: usize) -> Self {
        let s = Self {
            inner: Arc::new(Inner::new(min_threads)),
        };
        for _ in 0..min_threads {
            s.spawn(None);
        }
        s
    }

    // add_task, public interface for submitting a task,
    // executes a task, a new thread will be spawned in the event all threads are busy
    pub fn add_task(&self, task: Task) { 
        if self.inner.threads_waiting.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            self.spawn(Some(task))
        } else {
            let mut q = self.inner.task_list.lock().unwrap();
            q.push_back(task);
            self.inner.condvar.notify_one();
        }
    }

    // spawn a new thread
    fn spawn(&self, task: Option<Task>) {
        let inner = self.inner.clone();
        thread::spawn(move || {
            let inner = inner;
            // guard is dropped at end of context -- see drop impl
            let _liveness_guard = Register::new(&inner.total_threads);
            //println!("New Task: {:?}", inner.total_threads.load(std::sync::atomic::Ordering::Relaxed));
            if let Some(mut task) = task {
                task()
            }
            let mut tl = inner.task_list.lock().unwrap();
            loop {
                let mut task;
                loop {
                    // Try to pop a task and execute it
                    if let Some(t) = tl.pop_back() {
                        // We got a task, break out so we can execute,
                        // noitces this will also drop _waiting_guard if it has been created.
                        task = t;
                        break;
                    }
                    let _waiting_guard = Register::new(&inner.threads_waiting);
                    let timed_out = 
                        if inner.total_threads.load(std::sync::atomic::Ordering::Relaxed) <= inner.min_threads {
                            tl = inner.condvar.wait(tl).unwrap();
                            false
                        } else {
                            let (lock, timeout) = inner
                                .condvar
                                .wait_timeout(tl, Duration::from_millis(DEFAULT_THREAD_TIMEOUT))
                                .unwrap();
                            tl = lock;
                            timeout.timed_out()
                        };
                    // If we timed out and the 
                    if timed_out && tl.is_empty() {
                        //println!("Killing thread");
                        return
                    }
                };
                task()
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{thread, time};
    use std::sync::atomic::Ordering;
    #[test]
    fn test_create() {
        let tp = TaskPool::new(10); 
        println!("{:?}", tp);
        // sleep to let threads start
        thread::sleep(time::Duration::from_millis(500));
        println!("{:?}", tp);
        assert_eq!(10, tp.inner.threads_waiting.load(Ordering::Acquire));
        assert_eq!(10, tp.inner.total_threads.load(Ordering::Acquire));
    }

    #[test]
    fn test_thread_is_killed() {
        static THREAD_TIMEOUT_DURATION: time::Duration = time::Duration::from_millis(DEFAULT_THREAD_TIMEOUT + 500); 
        static PAUSE: time::Duration = time::Duration::from_millis(500);
        let tp = TaskPool::new(1);
        let t1 = Box::new(|| {
            for _ in 0..5 {
                println!("Thread 1 starting");
                thread::sleep(THREAD_TIMEOUT_DURATION);
                println!("Thread 1 finished, starting again")
            }
        });
        let t2 = Box::new(|| {
            println!("Thread 2 starting");
            thread::sleep(THREAD_TIMEOUT_DURATION);
            println!("Thread 2 finished")
        });
        tp.add_task(t1);
        thread::sleep(PAUSE);
        println!("{:?}", tp);
        assert_eq!(2, tp.inner.total_threads.load(Ordering::Relaxed), "first task started");
        tp.add_task(t2);
        thread::sleep(PAUSE);
        assert_eq!(2, tp.inner.total_threads.load(Ordering::Relaxed), "second task started");
        println!("{:?}", tp);
        thread::sleep(THREAD_TIMEOUT_DURATION * 2);
        assert_eq!(1, tp.inner.total_threads.load(Ordering::Relaxed), "second task should have died");
        println!("{:?}", tp);
    }
}
