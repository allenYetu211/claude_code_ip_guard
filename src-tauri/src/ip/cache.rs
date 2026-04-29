use std::time::{Duration, Instant};

use parking_lot::RwLock;

use crate::ip::IpReport;

pub struct ReportCache {
    inner: RwLock<Option<(Instant, IpReport)>>,
    ttl: RwLock<Duration>,
}

impl ReportCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            inner: RwLock::new(None),
            ttl: RwLock::new(default_ttl),
        }
    }

    pub fn set_ttl(&self, ttl: Duration) {
        *self.ttl.write() = ttl;
    }

    pub fn get_fresh(&self) -> Option<IpReport> {
        let ttl = *self.ttl.read();
        let guard = self.inner.read();
        guard.as_ref().and_then(|(at, r)| {
            if at.elapsed() < ttl {
                Some(r.clone())
            } else {
                None
            }
        })
    }

    pub fn put(&self, report: IpReport) {
        *self.inner.write() = Some((Instant::now(), report));
    }

    pub fn invalidate(&self) {
        *self.inner.write() = None;
    }
}
