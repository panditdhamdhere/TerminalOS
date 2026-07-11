use criterion::{Criterion, black_box, criterion_group, criterion_main};
use terminalos_config::parse_key_combo;

fn parse_keybinding_benchmark(c: &mut Criterion) {
    c.bench_function("parse_key_combo_ctrl_shift_tab", |b| {
        b.iter(|| parse_key_combo(black_box("Ctrl+Shift+Tab")))
    });
    c.bench_function("parse_key_combo_ctrl_q", |b| {
        b.iter(|| parse_key_combo(black_box("Ctrl+Q")))
    });
}

criterion_group!(benches, parse_keybinding_benchmark);
criterion_main!(benches);
