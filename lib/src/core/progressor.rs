use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

pub struct ProgressTracker<F> {
    progress: Progress,
    callback: F,
    report_interval: usize,
}

pub struct Progress {
    current: AtomicUsize,
    total: usize,
    message: String,
    completed: bool,
}

impl Progress {
    pub fn new() -> Self {
        Progress {
            current: AtomicUsize::new(0),
            total: 0,
            message: "".into(),
            completed: false,
        }
    }

    pub fn total(&mut self, total: usize) {
        self.total = total;
    }

    pub fn message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn reset(&mut self) {
        self.current.store(0, Ordering::Relaxed);
        self.total = 0;
        self.message.clear();
        self.completed = false;
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn display<F>(&self, formatter: F) -> String
    where
        F: FnOnce(usize, usize, String) -> String,
    {
        let current = self.current.load(Ordering::Relaxed);
        formatter(current, self.total, self.message.clone())
    }
}

pub trait ProgressReporter {
    fn report(&mut self);
    fn report_msg(&mut self, message: impl Into<String>);
    fn report_progress(&mut self, current: usize, total: usize);
    fn report_complete(&mut self);
}

impl<F> ProgressReporter for ProgressTracker<F>
where
    F: Fn(&Progress) + Send + Sync + 'static,
{
    fn report(&mut self) {
        (self.callback)(&self.progress);
        if self.progress.is_completed() {
            self.progress.reset();
        }
    }

    fn report_msg(&mut self, message: impl Into<String>) {
        self.progress.message(message);
        (self.callback)(&self.progress);
    }

    fn report_progress(&mut self, current: usize, total: usize) {
        self.progress.current.store(current, Ordering::Relaxed);
        self.progress.total(total);
        self.report();
    }

    fn report_complete(&mut self) {
        self.progress.completed = true;
        self.report();
    }
}

impl<F> ProgressReporter for Option<ProgressTracker<F>>
where
    F: Fn(&Progress) + Send + Sync + 'static,
{
    fn report(&mut self) {
        if let Some(tracker) = self {
            tracker.report();
        }
    }
    
    fn report_msg(&mut self, message: impl Into<String>) {
        if let Some(tracker) = self {
            tracker.report_msg(message);
        }
    }

    fn report_progress(&mut self, current: usize, total: usize) {
        if let Some(tracker) = self {
            tracker.report_progress(current, total);
        }
    }

    fn report_complete(&mut self) {
        if let Some(tracker) = self {
            tracker.report_complete();
        }
    }
}

impl<F> ProgressTracker<F> 
where
    F: Fn(&Progress) + Send + Sync + 'static,
{
    pub fn new(progress: Progress, callback: F) -> Self {
        let report_interval = (progress.total / 10).max(1).min(1000);  // 限制最大间隔
        Self {
            progress,
            callback,
            report_interval,
        }
    }

    pub fn interval(mut self, interval: usize) -> Self {
        self.report_interval = interval;
        self
    }

    pub fn update(&mut self, action: impl FnOnce()) {
        action();
        let curr = self.progress.current.fetch_add(1, Ordering::Relaxed) + 1;
        if curr % self.report_interval == 0 {
            self.report();
        }
        if curr == self.progress.total {
            self.report_complete();
        }
    }
}
