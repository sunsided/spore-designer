//! Where heavy work runs, so the window never freezes.
//!
//! On the desktop there are threads: [`Background::spawn`] starts one per job and
//! [`parallel_map`] spreads items over threads.
//!
//! A plain browser build (`wasm32-unknown-unknown`) has no threads. There, jobs wait in a
//! queue and [`Background::run_queued`] runs them at the start of each frame until a time
//! budget is spent, so pictures fill in over a few frames; [`parallel_map`] simply runs in
//! order. Jobs report back through channels either way, so callers do not care which it is.

/// How many threads parallel work may use (1 in the browser).
pub fn threads() -> usize {
    if cfg!(target_arch = "wasm32") {
        1
    } else {
        std::thread::available_parallelism().map_or(4, |n| n.get())
    }
}

/// `function` applied to every item, in parallel where there are threads; results in order.
pub fn parallel_map<T, R>(items: Vec<T>, function: impl Fn(T) -> R + Sync) -> Vec<R>
where
    T: Send,
    R: Send,
{
    if threads() == 1 {
        return items.into_iter().map(function).collect();
    }
    let function = &function;
    std::thread::scope(|scope| {
        let jobs: Vec<_> = items
            .into_iter()
            .map(|item| scope.spawn(move || function(item)))
            .collect();
        jobs.into_iter()
            .map(|job| job.join().expect("parallel job panicked"))
            .collect()
    })
}

/// Fire-and-forget jobs that report back on their own (through a channel).
#[derive(Default)]
pub struct Background {
    /// Jobs waiting for a frame's time budget (browser only).
    #[cfg(target_arch = "wasm32")]
    queue: std::collections::VecDeque<Box<dyn FnOnce()>>,
}

impl Background {
    /// Start `job`: on its own thread, or (in the browser) at the next frames.
    pub fn spawn(&mut self, job: impl FnOnce() + Send + 'static) {
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(job);
        #[cfg(target_arch = "wasm32")]
        self.queue.push_back(Box::new(job));
    }

    /// Run queued jobs until this frame's budget is spent (at least one, so work always
    /// moves on). Nothing to do where jobs have threads.
    pub fn run_queued(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            /// Work per frame, leaving room for drawing at 60 frames per second.
            const BUDGET: std::time::Duration = std::time::Duration::from_millis(12);
            let start = web_time::Instant::now();
            while let Some(job) = self.queue.pop_front() {
                job();
                if start.elapsed() >= BUDGET {
                    break;
                }
            }
        }
    }
}
