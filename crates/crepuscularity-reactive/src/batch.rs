use crate::effect::run_effect;
use crate::runtime::RUNTIME;

pub fn batch_begin() {
    RUNTIME.with(|rt| rt.borrow_mut().batch_depth += 1);
}

pub fn batch_end() {
    let should = RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        rt.batch_depth = rt.batch_depth.saturating_sub(1);
        rt.batch_depth == 0
    });
    if should {
        flush();
    }
}

pub(crate) fn maybe_flush() {
    let should = RUNTIME.with(|rt| rt.borrow().batch_depth == 0);
    if should {
        flush();
    }
}

pub fn flush() {
    loop {
        let effects: Vec<u32> =
            RUNTIME.with(|rt| std::mem::take(&mut rt.borrow_mut().pending_effects));
        if effects.is_empty() {
            break;
        }
        for id in effects {
            run_effect(id);
        }
    }
}
