use std::cell::RefCell;
use std::rc::Rc;

pub struct ReentranceGuard {
    counter: Rc<RefCell<u32>>,
}

pub struct ReentranceGuardLock<'a> {
    guard: &'a ReentranceGuard,
}

impl ReentranceGuard {
    pub fn new() -> ReentranceGuard {
        ReentranceGuard {
            counter: Rc::new(RefCell::new(0)),
        }
    }

    pub fn try_lock(&self) -> Option<ReentranceGuardLock> {
        let mut c = self.counter.borrow_mut();
        if *c == 0 {
            *c += 1;
            assert!(*c == 1);
            let lock = ReentranceGuardLock { guard: self };
            Some(lock)
        } else {
            None
        }
    }

    fn drop_lock(&self) {
        let mut c = self.counter.borrow_mut();
        *c -= 1;
        assert!(*c == 0);
    }
}

impl<'a> Drop for ReentranceGuardLock<'a> {
    fn drop(&mut self) {
        self.guard.drop_lock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accure_lock() {
        let guard = ReentranceGuard::new();
        let lock = guard.try_lock();

        assert!(lock.is_some());
    }

    #[test]
    fn deny_second_lock() {
        let guard = ReentranceGuard::new();
        let lock1 = guard.try_lock();
        let lock2 = guard.try_lock();
        assert!(lock2.is_none());
    }

    #[test]
    fn drop_lock() {
        let guard = ReentranceGuard::new();
        if let Some(lock1) = guard.try_lock() {}
        let lock2 = guard.try_lock();
        assert!(lock2.is_some());
    }
}
