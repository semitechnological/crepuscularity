//! Flexbox-style layout for [`EmbeddedDocument`] trees.

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use crate::document::{Align, EmbeddedDocument, EmbeddedNode, FlexDir, Justify, Rect, SizeHint};
use crate::font::FontMetrics;
use crate::screen::ScreenSize;

pub fn layout_tree(root: &mut EmbeddedNode, screen: ScreenSize) {
    root.bounds = Rect::new(0, 0, screen.width, screen.height);
    layout_children(root);
}

pub fn layout_document(doc: &mut EmbeddedDocument) {
    for node in &mut doc.root {
        layout_tree(node, doc.screen);
    }
    doc.reindex_by_id();
}

fn layout_children(parent: &mut EmbeddedNode) {
    if let Some(text) = &parent.text {
        let metrics = FontMetrics::default();
        let (tw, th) = metrics.measure_text(text);
        let pad_x = parent.style.pad_x();
        let pad_y = parent.style.pad_y();
        parent.bounds.w = tw.saturating_add(pad_x * 2).min(parent.bounds.w);
        parent.bounds.h = th.saturating_add(pad_y * 2).min(parent.bounds.h);
        return;
    }
    if parent.children.is_empty() {
        return;
    }

    let pad_x = parent.style.pad_x();
    let pad_y = parent.style.pad_y();
    let inner = Rect::new(
        parent.bounds.x.saturating_add(pad_x),
        parent.bounds.y.saturating_add(pad_y),
        parent.bounds.w.saturating_sub(pad_x * 2),
        parent.bounds.h.saturating_sub(pad_y * 2),
    );
    let gap = parent.style.gap;
    let row = parent.style.flex_dir == FlexDir::Row;

    let main_sizes = compute_main_sizes(&parent.children, row, gap, inner);
    let offsets = compute_offsets(&main_sizes, parent.style.justify, gap, inner, row);

    for (i, child) in parent.children.iter_mut().enumerate() {
        let cross = cross_size(child, row, inner);
        let (x, y, w, h) = if row {
            (
                inner.x.saturating_add(offsets[i]),
                align_cross(child, inner.y, cross, inner.h, parent.style.align),
                main_sizes[i],
                cross,
            )
        } else {
            (
                align_cross(child, inner.x, cross, inner.w, parent.style.align),
                inner.y.saturating_add(offsets[i]),
                cross,
                main_sizes[i],
            )
        };
        child.bounds = Rect::new(x, y, w, h);
        layout_children(child);
    }
}

fn compute_main_sizes(children: &[EmbeddedNode], row: bool, gap: u16, inner: Rect) -> Vec<u16> {
    let n = children.len();
    let mut main_sizes: Vec<u16> = children
        .iter()
        .map(|c| desired_main(c, row, inner))
        .collect();

    let mut flex_count = 0usize;
    let mut fixed_sum = 0u16;
    for (i, child) in children.iter().enumerate() {
        let hint = main_hint(child, row);
        if hint == SizeHint::Flex1 || hint == SizeHint::Fill {
            flex_count += 1;
        } else {
            fixed_sum = fixed_sum.saturating_add(main_sizes[i]);
        }
    }
    if flex_count > 0 {
        let gaps = gap.saturating_mul((n.saturating_sub(1)) as u16);
        let avail_main = inner_main(inner, row)
            .saturating_sub(fixed_sum)
            .saturating_sub(gaps);
        let each = avail_main / flex_count as u16;
        for (i, child) in children.iter().enumerate() {
            let hint = main_hint(child, row);
            if hint == SizeHint::Flex1 || hint == SizeHint::Fill {
                main_sizes[i] = each.max(1);
            }
        }
    }

    main_sizes
}

fn compute_offsets(
    main_sizes: &[u16],
    justify: Justify,
    gap: u16,
    inner: Rect,
    row: bool,
) -> Vec<u16> {
    let n = main_sizes.len();
    let total_main: u16 =
        main_sizes.iter().sum::<u16>() + gap.saturating_mul((n.saturating_sub(1)) as u16);
    let free_main = inner_main(inner, row).saturating_sub(total_main);

    let (mut pos, extra_gap) = match justify {
        Justify::Center if free_main > 0 => (free_main / 2, 0),
        Justify::Between if n > 1 && free_main > 0 => (0, free_main / (n - 1) as u16),
        Justify::End if free_main > 0 => (free_main, 0),
        _ => (0, 0),
    };

    main_sizes
        .iter()
        .map(|&size| {
            let current = pos;
            pos = pos
                .saturating_add(size)
                .saturating_add(gap)
                .saturating_add(extra_gap);
            current
        })
        .collect()
}

fn inner_main(inner: Rect, row: bool) -> u16 {
    if row {
        inner.w
    } else {
        inner.h
    }
}

fn main_hint(child: &EmbeddedNode, row: bool) -> SizeHint {
    if row {
        child.style.width
    } else {
        child.style.height
    }
}

fn desired_main(child: &EmbeddedNode, row: bool, inner: Rect) -> u16 {
    if let Some(text) = &child.text {
        let metrics = FontMetrics::default();
        let (_, th) = metrics.measure_text(text);
        let pad = child.style.pad_y();
        return th.saturating_add(pad * 2).min(inner_main(inner, row));
    }
    let hint = main_hint(child, row);
    let avail = inner_main(inner, row);
    match hint {
        SizeHint::Fixed(v) => v.min(avail),
        SizeHint::Fill | SizeHint::Flex1 => avail,
        SizeHint::Fraction { num, den } => {
            if den == 0 {
                avail
            } else {
                ((avail as u32).saturating_mul(num as u32) / den as u32).min(u16::MAX as u32) as u16
            }
        }
        SizeHint::Auto => {
            if row {
                32u16.min(avail)
            } else {
                16u16.min(avail)
            }
        }
    }
}

fn cross_size(child: &EmbeddedNode, row: bool, inner: Rect) -> u16 {
    let hint = if row {
        child.style.height
    } else {
        child.style.width
    };
    let avail = if row { inner.h } else { inner.w };
    match hint {
        SizeHint::Fixed(v) => v.min(avail),
        SizeHint::Fill | SizeHint::Flex1 => avail,
        SizeHint::Fraction { num, den } => {
            if den == 0 {
                avail
            } else {
                ((avail as u32).saturating_mul(num as u32) / den as u32).min(u16::MAX as u32) as u16
            }
        }
        SizeHint::Auto => {
            if let Some(text) = &child.text {
                let metrics = FontMetrics::default();
                let (tw, _) = metrics.measure_text(text);
                return tw.saturating_add(child.style.pad_x() * 2).min(avail);
            }
            avail
        }
    }
}

fn align_cross(_child: &EmbeddedNode, start: u16, size: u16, avail: u16, align: Align) -> u16 {
    match align {
        Align::Center => start.saturating_add(avail.saturating_sub(size) / 2),
        Align::End => start.saturating_add(avail.saturating_sub(size)),
        _ => start,
    }
}
