use std::time::Instant;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct PerfStats {
    pub lex_time_ns: AtomicU64,
    pub parse_time_ns: AtomicU64,
    pub expand_time_ns: AtomicU64,
    pub execute_time_ns: AtomicU64,
    pub count: AtomicUsize,
}

impl PerfStats {
    pub const fn new() -> Self {
        Self {
            lex_time_ns: AtomicU64::new(0),
            parse_time_ns: AtomicU64::new(0),
            expand_time_ns: AtomicU64::new(0),
            execute_time_ns: AtomicU64::new(0),
            count: AtomicUsize::new(0),
        }
    }

    pub fn record_lex(&self, elapsed_ns: u64) {
        self.lex_time_ns.fetch_add(elapsed_ns, Ordering::Relaxed);
    }

    pub fn record_parse(&self, elapsed_ns: u64) {
        self.parse_time_ns.fetch_add(elapsed_ns, Ordering::Relaxed);
    }

    pub fn record_expand(&self, elapsed_ns: u64) {
        self.expand_time_ns.fetch_add(elapsed_ns, Ordering::Relaxed);
    }

    pub fn record_execute(&self, elapsed_ns: u64) {
        self.execute_time_ns.fetch_add(elapsed_ns, Ordering::Relaxed);
    }

    pub fn increment_count(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn print_report(&self) {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return;
        }

        let lex_ns = self.lex_time_ns.load(Ordering::Relaxed);
        let parse_ns = self.parse_time_ns.load(Ordering::Relaxed);
        let expand_ns = self.expand_time_ns.load(Ordering::Relaxed);
        let execute_ns = self.execute_time_ns.load(Ordering::Relaxed);
        let total_ns = lex_ns + parse_ns + expand_ns + execute_ns;

        eprintln!("\n📊 AUSH Performance Stats ({} commands):", count);
        eprintln!("  Lex:     {:6.2}µs ({:5.1}%)", lex_ns as f64 / count as f64 / 1000.0, lex_ns as f64 / total_ns as f64 * 100.0);
        eprintln!("  Parse:   {:6.2}µs ({:5.1}%)", parse_ns as f64 / count as f64 / 1000.0, parse_ns as f64 / total_ns as f64 * 100.0);
        eprintln!("  Expand:  {:6.2}µs ({:5.1}%)", expand_ns as f64 / count as f64 / 1000.0, expand_ns as f64 / total_ns as f64 * 100.0);
        eprintln!("  Execute: {:6.2}µs ({:5.1}%)", execute_ns as f64 / count as f64 / 1000.0, execute_ns as f64 / total_ns as f64 * 100.0);
        eprintln!("  Total:   {:6.2}µs per command", total_ns as f64 / count as f64 / 1000.0);
    }

    pub fn reset(&self) {
        self.lex_time_ns.store(0, Ordering::Relaxed);
        self.parse_time_ns.store(0, Ordering::Relaxed);
        self.expand_time_ns.store(0, Ordering::Relaxed);
        self.execute_time_ns.store(0, Ordering::Relaxed);
        self.count.store(0, Ordering::Relaxed);
    }
}

pub static PERF_STATS: PerfStats = PerfStats::new();

pub struct Timer {
    start: Instant,
    phase: &'static str,
}

impl Timer {
    pub fn new(phase: &'static str) -> Self {
        Self {
            start: Instant::now(),
            phase,
        }
    }

    pub fn finish(self) {
        let elapsed_ns = self.start.elapsed().as_nanos() as u64;
        match self.phase {
            "lex" => PERF_STATS.record_lex(elapsed_ns),
            "parse" => PERF_STATS.record_parse(elapsed_ns),
            "expand" => PERF_STATS.record_expand(elapsed_ns),
            "execute" => PERF_STATS.record_execute(elapsed_ns),
            _ => {}
        }
    }
}

#[macro_export]
macro_rules! time_phase {
    ($phase:expr, $code:expr) => {{
        let _timer = $crate::perf::Timer::new($phase);
        let result = $code;
        _timer.finish();
        result
    }};
}
