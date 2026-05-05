use std::cell::RefCell;
use std::rc::Rc;

use crepuscularity_reactive::{batch_begin, batch_end, Effect, Memo, Signal};

fn main() {
    let count = Signal::new(0);
    let doubled = {
        let count = count.clone();
        Memo::new(move || count.get() * 2)
    };

    let label = Rc::new(RefCell::new(String::new()));
    let effect = {
        let doubled = doubled.clone();
        let label = Rc::clone(&label);
        Effect::new(move || {
            *label.borrow_mut() = format!("doubled={}", doubled.get());
        })
    };

    batch_begin();
    count.set(1);
    count.set(2);
    batch_end();

    assert_eq!(label.borrow().as_str(), "doubled=4");

    effect.dispose();
    doubled.dispose();
}
