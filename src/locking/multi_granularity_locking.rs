use crate::locking::*;
use crate::locking::LockingError::LockNotHeld;
use crate::locking::MGLLock::{Denied, IntentionRead, IntentionWrite, Read, Write};

/// The underlying locktree that will implement the MultiGranularityLocking trait
///
/// # Attention!
/// Do not rename this struct, it gets instantiated in the tests!
pub struct MultiGranularityLockTree {
    root: MGLNode,
    // Add more members here if necessary
}

impl MultiGranularityLockTree {
    /// Construct a multi-granularity lock tree with initialized struct members.
    /// The root node here is never part of the path in the get_state and lock functions.
    pub fn new(root: MGLNode) -> Self {
        Self { root }
    }
}

impl MultiGranularityLocking for MultiGranularityLockTree {
    /// See documentation in ./mod.rs, and check exercise and lecture slides.
    fn get_state(&mut self, path: Vec<&str>) -> &mut MGLNode {
        let mut current_pos = &mut self.root;
        for string in path {
            let children = current_pos.children.get_mut(string);
            current_pos = children.unwrap();
        }

        current_pos
    }

    /// See documentation in ./mod.rs, and check exercise and lecture slides.
    fn lock(&mut self, tid: TID, path: Vec<&str>, req: LockRequest) -> Vec<MGLLock> {
        let mut current_pos = &mut self.root;
        let mut idx = 0;
        let len = path.len();
        let mut mgl_lock_current;
        let mut denied = false;
        let mut vec:Vec<MGLLock> = Vec::new();
        for string in path {
            let children = current_pos.children.get_mut(string);
            current_pos = children.unwrap();
            idx += 1;

            if idx == len {
                if req == LockRequest::Read {
                    mgl_lock_current = Read;
                } else {
                    mgl_lock_current = Write;
                }
            } else {
                if req == LockRequest::Read {
                    mgl_lock_current = IntentionRead;
                } else {
                    mgl_lock_current = IntentionWrite;
                }
            }

            denied = false;
            for (tid_val, mgl_lock) in current_pos.locks.iter() {
                if mgl_lock_current == Read {
                    if *mgl_lock == Write || *mgl_lock == IntentionWrite {
                        denied = true;
                    }
                } else if mgl_lock_current == Write {
                    if *mgl_lock == Write || *mgl_lock == IntentionWrite ||
                        *mgl_lock == IntentionRead || *mgl_lock == Read {
                        denied = true;
                    }
                } else if mgl_lock_current == IntentionRead {
                    if *mgl_lock == Write {
                        denied = true;
                    }
                } else if mgl_lock_current == IntentionWrite {
                    if *mgl_lock == Write || *mgl_lock == Read {
                        denied = true;
                    }
                }
            }

            if denied {
                vec.push(MGLLock::Denied);
                break;
            } else {
                current_pos.locks.insert((tid, mgl_lock_current.clone()));
                vec.push(mgl_lock_current.clone());
            }
        }

        if denied {
            self.release(tid);
        }

        vec
    }

    /// See documentation in ./mod.rs, and check exercise and lecture slides.
    fn release(&mut self, tid: TID) -> Result<(), LockingError> {
        let mut idx_err = 0;
        for (string, node) in self.root.children.iter_mut() {
            if node.locks.contains(&(tid, Read)) || node.locks.contains(&(tid, Write)) ||
                node.locks.contains(&(tid, IntentionWrite)) || node.locks.contains(&(tid, IntentionRead)) {
                release_rec(node, tid);
            } else {
                idx_err += 1;
            }
        }

        if idx_err == self.root.children.len() {
            return Err(LockingError::LockNotHeld);
        }

        Ok(())
    }
}

fn release_rec(node: &mut MGLNode, tid: TID) {
    node.locks.remove(&(tid, Read));
    node.locks.remove(&(tid, Write));
    node.locks.remove(&(tid, IntentionRead));
    node.locks.remove(&(tid, IntentionWrite));

    for (string, children) in node.children.iter_mut() {
        if children.locks.contains(&(tid, Read)) || children.locks.contains(&(tid, Write)) ||
            children.locks.contains(&(tid, IntentionWrite)) || children.locks.contains(&(tid, IntentionRead)) {
            release_rec(children, tid);
        }
    }
}
