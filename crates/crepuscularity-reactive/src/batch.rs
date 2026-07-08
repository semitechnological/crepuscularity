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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::RUNTIME;

    #[test]
    fn test_batch_begin_increments_depth() {
        let initial_depth = RUNTIME.with(|rt| rt.borrow().batch_depth);
        batch_begin();
        let depth_after_begin = RUNTIME.with(|rt| rt.borrow().batch_depth);
        assert_eq!(depth_after_begin, initial_depth + 1);

        batch_end();
        let depth_after_end = RUNTIME.with(|rt| rt.borrow().batch_depth);
        assert_eq!(depth_after_end, initial_depth);
    }

    #[test]
    fn test_nested_batches() {
        let initial_depth = RUNTIME.with(|rt| rt.borrow().batch_depth);

        batch_begin();
        batch_begin();

        let depth_nested = RUNTIME.with(|rt| rt.borrow().batch_depth);
        assert_eq!(depth_nested, initial_depth + 2);

        batch_end();
        assert_eq!(RUNTIME.with(|rt| rt.borrow().batch_depth), initial_depth + 1);

        batch_end();
        assert_eq!(RUNTIME.with(|rt| rt.borrow().batch_depth), initial_depth);
    }

    #[test]
    fn test_batch_end_no_underflow() {
        // Ensure depth is 0
        RUNTIME.with(|rt| rt.borrow_mut().batch_depth = 0);
        batch_end();
        let depth_after = RUNTIME.with(|rt| rt.borrow().batch_depth);
        assert_eq!(depth_after, 0);
    }

    #[test]
    fn test_batch_defers_effects() {
        use std::rc::Rc;
        use std::cell::Cell;
        use crate::signal::Signal;
        use crate::effect::Effect;

        let signal = Signal::new(0);
        let run_count = Rc::new(Cell::new(0));
        let run_count_clone = run_count.clone();

        let signal_clone = signal.clone();
        let _effect = Effect::new(move || {
            signal_clone.get();
            run_count_clone.set(run_count_clone.get() + 1);
        });

        // Initial run happens immediately
        assert_eq!(run_count.get(), 1);

        batch_begin();
        signal.set(1);
        signal.set(2);

        // Effects should not run while in a batch
        assert_eq!(run_count.get(), 1);

        batch_end();

        // Effects run once after batch ends
        assert_eq!(run_count.get(), 2);
    }
}
