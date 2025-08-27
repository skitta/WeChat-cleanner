use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// 进度回调 trait，用于接收进度更新
pub trait ProgressCallback: Send + Sync {
    /// 接收进度更新
    fn on_progress(&self, progress: &Progress);
}

/// 进度追踪器，使用 trait object 简化泛型设计
pub struct ProgressTracker {
    progress: Progress,
    callback: Box<dyn ProgressCallback>,
    report_interval: usize,
}

/// 进度信息结构
pub struct Progress {
    current: AtomicUsize,
    total: usize,
    message: String,
    completed: bool,
}

impl Progress {
    /// 创建新的进度实例
    pub fn new() -> Self {
        Progress {
            current: AtomicUsize::new(0),
            total: 0,
            message: String::new(),
            completed: false,
        }
    }

    /// 设置总数
    pub fn set_total(&mut self, total: usize) {
        self.total = total;
    }

    /// 设置消息
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// 重置进度
    pub fn reset(&mut self) {
        self.current.store(0, Ordering::Relaxed);
        self.total = 0;
        self.message.clear();
        self.completed = false;
    }

    /// 获取当前值
    pub fn current(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }

    /// 获取总数
    pub fn total(&self) -> usize {
        self.total
    }

    /// 获取消息
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 是否已完成
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// 格式化显示进度
    pub fn display<F>(&self, formatter: F) -> String
    where
        F: FnOnce(usize, usize, &str) -> String,
    {
        let current = self.current.load(Ordering::Relaxed);
        formatter(current, self.total, &self.message)
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// 进度报告 trait，用于统一进度更新接口
pub trait ProgressReporter {
    /// 报告当前进度
    fn report(&mut self);
    /// 报告消息
    fn report_msg(&mut self, message: impl Into<String>);
    /// 报告进度和总数
    fn report_progress(&mut self, current: usize, total: usize);
    /// 报告完成状态
    fn report_complete(&mut self);
}

impl ProgressReporter for ProgressTracker {
    fn report(&mut self) {
        self.callback.on_progress(&self.progress);
        if self.progress.is_completed() {
            self.progress.reset();
        }
    }

    fn report_msg(&mut self, message: impl Into<String>) {
        self.progress.set_message(message);
        self.callback.on_progress(&self.progress);
    }

    fn report_progress(&mut self, current: usize, total: usize) {
        self.progress.current.store(current, Ordering::Relaxed);
        self.progress.set_total(total);
        self.report();
    }

    fn report_complete(&mut self) {
        self.progress.completed = true;
        self.report();
    }
}

impl ProgressReporter for Option<ProgressTracker> {
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

impl ProgressTracker {
    /// 创建新的进度追踪器
    pub fn new(progress: Progress, callback: impl ProgressCallback + 'static) -> Self {
        let report_interval = (progress.total / 10).max(1).min(1000); // 限制最大间隔
        Self {
            progress,
            callback: Box::new(callback),
            report_interval,
        }
    }

    /// 设置报告间隔
    pub fn with_interval(mut self, interval: usize) -> Self {
        self.report_interval = interval;
        self
    }

    /// 更新进度并执行动作
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

// 为闭包实现 ProgressCallback trait，保持向后兼容
impl<F> ProgressCallback for F
where
    F: Fn(&Progress) + Send + Sync,
{
    fn on_progress(&self, progress: &Progress) {
        self(progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestCallback {
        messages: Arc<Mutex<Vec<String>>>,
    }

    impl TestCallback {
        fn new() -> (Self, Arc<Mutex<Vec<String>>>) {
            let messages = Arc::new(Mutex::new(Vec::new()));
            (Self { messages: Arc::clone(&messages) }, messages)
        }
    }

    impl ProgressCallback for TestCallback {
        fn on_progress(&self, progress: &Progress) {
            let msg = format!("{}:{}/{}", progress.message(), progress.current(), progress.total());
            self.messages.lock().unwrap().push(msg);
        }
    }

    #[test]
    fn test_progress_tracker() {
        let (callback, messages) = TestCallback::new();
        let mut tracker = ProgressTracker::new(Progress::new(), callback);
        
        tracker.report_msg("Starting");
        tracker.report_progress(5, 10);
        tracker.report_complete();
        
        let msgs = messages.lock().unwrap();
        assert!(msgs.len() >= 2);
        assert!(msgs[0].contains("Starting"));
    }

    #[test]
    fn test_closure_compatibility() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let messages_clone = Arc::clone(&messages);
        
        let callback = move |progress: &Progress| {
            messages_clone.lock().unwrap().push(progress.message().to_string());
        };
        
        let mut tracker = ProgressTracker::new(Progress::new(), callback);
        tracker.report_msg("Test message");
        
        assert_eq!(messages.lock().unwrap()[0], "Test message");
    }
}
