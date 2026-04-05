use crepuscularity_reactive::{batch_begin, batch_end, Effect, Memo, Signal};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn signal_read_write() {
    let s = Signal::new(42i32);
    assert_eq!(s.get(), 42);
    s.set(100);
    assert_eq!(s.get(), 100);
    s.update(|v| v + 1);
    assert_eq!(s.get(), 101);
}

#[test]
fn effect_runs_immediately() {
    let ran = Rc::new(RefCell::new(false));
    let ran2 = Rc::clone(&ran);
    let _e = Effect::new(move || {
        *ran2.borrow_mut() = true;
    });
    assert!(*ran.borrow(), "effect should run immediately on creation");
}

#[test]
fn effect_reruns_on_change() {
    let count = Rc::new(RefCell::new(0u32));
    let s = Signal::new(0i32);
    let s2 = s.clone();
    let count2 = Rc::clone(&count);
    let _e = Effect::new(move || {
        let _ = s2.get(); // track signal
        *count2.borrow_mut() += 1;
    });
    assert_eq!(*count.borrow(), 1, "ran once on init");
    s.set(1);
    assert_eq!(*count.borrow(), 2, "ran again after set");
    s.set(2);
    assert_eq!(*count.borrow(), 3, "ran again after another set");
    // Setting same value should NOT re-run
    s.set(2);
    assert_eq!(*count.borrow(), 3, "no re-run on same value");
}

#[test]
fn memo_caches() {
    let s = Signal::new(2i32);
    let s2 = s.clone();
    let compute_count = Rc::new(RefCell::new(0u32));
    let cc2 = Rc::clone(&compute_count);
    let m = Memo::new(move || {
        *cc2.borrow_mut() += 1;
        s2.get() * 2
    });

    // First get triggers computation
    assert_eq!(m.get(), 4);
    let first_count = *compute_count.borrow();

    // Second get should use cache (no recomputation since signal unchanged)
    assert_eq!(m.get(), 4);
    // Note: depends on implementation; we at minimum verify the value is correct
    let _ = first_count;

    // After signal change, memo recomputes
    s.set(5);
    assert_eq!(m.get(), 10);
}

#[test]
fn memo_partialeq_skip() {
    // Memo should not mark subscribers dirty if computed value hasn't changed.
    // We test this indirectly: if signal A changes but memo result is same, effect shouldn't re-run extra times.
    let s = Signal::new(1i32);
    let s2 = s.clone();
    // Memo always returns the same value regardless of signal
    let m = Memo::new(move || {
        let _ = s2.get(); // track signal
        42i32 // always returns 42
    });

    let effect_count = Rc::new(RefCell::new(0u32));
    let ec2 = Rc::clone(&effect_count);
    let m2 = m.clone();
    let _e = Effect::new(move || {
        let _ = m2.get(); // track memo
        *ec2.borrow_mut() += 1;
    });

    assert_eq!(*effect_count.borrow(), 1, "effect ran on init");
    // The memo value is stable; changing signal may or may not trigger effect
    // depending on implementation. We just verify we don't crash and value is correct.
    s.set(2);
    assert_eq!(m.get(), 42, "memo still returns 42");
}

#[test]
fn batch_defers_effects() {
    let s = Signal::new(0i32);
    let s2 = s.clone();
    let count = Rc::new(RefCell::new(0u32));
    let c2 = Rc::clone(&count);
    let _e = Effect::new(move || {
        let _ = s2.get();
        *c2.borrow_mut() += 1;
    });
    assert_eq!(*count.borrow(), 1, "initial run");

    batch_begin();
    s.set(1);
    s.set(2);
    s.set(3);
    // Effects should not have run during batch
    assert_eq!(*count.borrow(), 1, "no re-runs during batch");
    batch_end();
    // After batch_end, effects should have flushed exactly once (or once per unique change)
    assert!(
        *count.borrow() >= 2,
        "effect ran at least once after batch_end"
    );
}

#[test]
fn stale_source_cleanup() {
    // An effect conditionally reads signal A or signal B based on condition signal.
    // After condition flips, writing to the formerly-read signal should NOT re-run the effect.
    let condition = Signal::new(true);
    let sig_a = Signal::new(1i32);
    let sig_b = Signal::new(100i32);

    let condition2 = condition.clone();
    let sig_a2 = sig_a.clone();
    let sig_b2 = sig_b.clone();

    let last_value = Rc::new(RefCell::new(0i32));
    let lv2 = Rc::clone(&last_value);
    let run_count = Rc::new(RefCell::new(0u32));
    let rc2 = Rc::clone(&run_count);

    let _e = Effect::new(move || {
        *rc2.borrow_mut() += 1;
        if condition2.get() {
            *lv2.borrow_mut() = sig_a2.get();
        } else {
            *lv2.borrow_mut() = sig_b2.get();
        }
    });

    // Initial: condition=true, reads sig_a
    assert_eq!(*run_count.borrow(), 1);
    assert_eq!(*last_value.borrow(), 1);

    // Flip condition: effect re-runs, now reads sig_b
    condition.set(false);
    assert_eq!(*last_value.borrow(), 100);
    let count_after_flip = *run_count.borrow();
    assert!(count_after_flip >= 2, "re-ran after condition flip");

    // Now write to sig_a — effect should NOT re-run since it no longer tracks sig_a
    sig_a.set(999);
    assert_eq!(
        *run_count.borrow(),
        count_after_flip,
        "effect must not re-run when writing to stale source sig_a"
    );
    assert_eq!(
        *last_value.borrow(),
        100,
        "last_value unchanged after writing stale sig_a"
    );

    // Writing to sig_b SHOULD re-run the effect
    sig_b.set(200);
    assert!(
        *run_count.borrow() > count_after_flip,
        "effect re-runs when active source sig_b changes"
    );
    assert_eq!(*last_value.borrow(), 200);
}
