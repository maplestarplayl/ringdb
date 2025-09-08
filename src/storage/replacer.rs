use std::sync::Mutex;

pub struct ClockReplacer {
    inner: Mutex<ClockReplacerInner>,
}
struct ClockReplacerInner {
    frames: Vec<FrameState>,
    clock_hand: usize,
    capacity: usize,
}
struct FrameState {
    is_pinned: bool,
    ref_bit: bool,
}

impl ClockReplacer {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(ClockReplacerInner {
                frames: (0..capacity)
                    .map(|_| FrameState {
                        is_pinned: true,
                        ref_bit: false,
                    })
                    .collect(),
                clock_hand: 0,
                capacity,
            }),
        }
    }
    pub fn victim(&self) -> Option<usize> {
        let mut inner = self.inner.lock().unwrap();
        if inner.capacity == 0 {
            return None;
        }
        for _ in 0..(2 * inner.capacity) {
            let hand = inner.clock_hand;
            let frame_state = &mut inner.frames[hand];
            if !frame_state.is_pinned {
                if frame_state.ref_bit {
                    frame_state.ref_bit = false;
                } else {
                    return Some(hand);
                }
            }
            inner.clock_hand = (inner.clock_hand + 1) % inner.capacity;
        }
        None
    }
    pub fn pin(&self, frame_id: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.frames[frame_id].is_pinned = true;
    }
    pub fn unpin(&self, frame_id: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.frames[frame_id].is_pinned = false;
        inner.frames[frame_id].ref_bit = true;
    }
}
