use crate::RowID;
use crate::locking::*;
use crate::locking::BasicLockResult::{Granted, Queued};
use crate::locking::LockingError::{LockAlreadyHeld, LockNotHeld};
use crate::locking::LockRequest::{Read, Write};

/// The underlying locktable that will implement the BasicLocking trait:
/// Do not rename this struct, it gets instantiated in the tests!
pub struct BasicLockTable {
    // Add the necessary members here
    hash_map: HashMap<RowID, BasicLockState>
}

impl BasicLockTable {
    /// Construct a locktable with initialized struct members:
    pub fn new() -> Self {
        Self {
            hash_map: HashMap::new()
        }
    }
}

impl BasicLocking for BasicLockTable {
    /// See documentation in ./mod.rs, and check exercise and lecture slides.
    fn get_state(&self, object: RowID) -> Option<&BasicLockState> {
        self.hash_map.get(&object)
    }

    /// See documentation in ./mod.rs, and check exercise and lecture slides.
    fn lock(
        &mut self,
        tid: TID,
        object: RowID,
        req: LockRequest,
    ) -> Result<BasicLockResult, LockingError> {
        let state_op = self.hash_map.get_mut(&object);
        if state_op.is_none() {
            let mut new_state = BasicLockState::default();
            if req == Read {
                new_state.readers.insert(tid);
                new_state.writer = Option::None;
            } else {
                new_state.writer = Option::Some(tid);
            }
            self.hash_map.insert(object, new_state);

            return Result::Ok(Granted)
        }

        let mut state = state_op.unwrap();

        if req == Read {
            if state.readers.contains(&tid) {
                return Result::Err(LockAlreadyHeld)
            }

            if state.writer == None {
                state.readers.insert(tid);
                return Result::Ok(Granted)
            } else {
                if state.writer.unwrap() == tid {
                    return Result::Err(LockAlreadyHeld)
                }
                state.queue.push_back((tid, req));
                return Result::Ok(Queued)
            }
        } else {
            if state.writer == Some(tid) {
                return Result::Err(LockAlreadyHeld)
            }

            if state.writer == None && state.readers.len() == 1 && state.readers.contains(&tid) {
                state.readers.clear();
                state.writer = Some(tid);
                return Result::Ok(Granted)
            }

            if state.writer == None && state.readers.is_empty() {
                state.writer = Some(tid);
                return Result::Ok(Granted)
            } else {
                state.queue.push_back((tid, req));
                return Result::Ok(Queued)
            }
        }
    }

    /// See documentation in ./mod.rs, and check exercise and lecture slides.
    fn release(
        &mut self,
        tid: TID,
        object: RowID,
    ) -> Result<Vec<(TID, LockRequest)>, LockingError> {
        let mut vec: Vec<(TID, LockRequest)> = Vec::new();

        let state = self.hash_map.get_mut(&object);
        if state.is_none() {
            return Result::Err(LockNotHeld)
        }

        let state = state.unwrap();

        if state.writer != Some(tid) && !state.readers.contains(&tid) {
            return Result::Err(LockNotHeld)
        }

        state.writer = None;
        state.readers.remove(&tid);

        let mut bool_var = true;

        while bool_var {
            let elem = state.queue.pop_front();
            if elem.is_none() {
                return Result::Ok(vec)
            }

            let val = elem.unwrap();
            bool_var = false;
            if val.1 == Read {
                if state.writer == None {
                    bool_var = true;
                    state.readers.insert(val.0);
                    vec.push((val.0.clone(), val.1.clone()));
                }
            } else {
                if state.writer == None && state.readers.len() == 1 && state.readers.contains(&val.0.clone()) {
                    if vec.contains(&(val.0.clone(), Read)) {
                        vec.pop();
                    }
                    bool_var = true;
                    state.readers.clear();
                    state.writer = Some(val.0.clone());
                    vec.push((val.0.clone(), val.1.clone()));
                } else if state.writer == None && state.readers.is_empty() {
                    bool_var = true;
                    state.writer = Some(val.0.clone());
                    vec.push((val.0.clone(), val.1.clone()));
                }
            }

            if bool_var == false {
                state.queue.push_back((val.0.clone(), val.1.clone()));
            }
        }

        Ok(vec)
    }
}
