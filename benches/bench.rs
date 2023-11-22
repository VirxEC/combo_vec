use combo_vec::{combo_vec, re_arr, ComboVec, ReArr};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const MY_VEC: ComboVec<i32, 8> = combo_vec![];
const MY_ARR: ReArr<i32, 8> = re_arr![];

fn new(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(|| black_box(ComboVec::<i32, 3>::new())));
}

fn new_from_arr(c: &mut Criterion) {
    c.bench_function("new_macro", |b| b.iter(|| black_box(combo_vec![1, 2, 3])));
}

fn push(c: &mut Criterion) {
    c.bench_function("push", |b| {
        b.iter(|| {
            let mut my_arr = black_box(combo_vec![i32; 8]);
            my_arr.push(4);
        })
    });
}

fn push_no_vec(c: &mut Criterion) {
    c.bench_function("push_no_vec", |b| {
        b.iter(|| {
            let mut my_arr = black_box(re_arr![i32; 8]);
            my_arr.push(4);
        })
    });
}

fn push_clone_const_no_vec(c: &mut Criterion) {
    c.bench_function("push_clone_const_no_vec", |b| {
        b.iter(|| {
            let mut my_arr = black_box(MY_ARR);
            my_arr.push(4);
        })
    });
}

fn push_clone_const(c: &mut Criterion) {
    c.bench_function("push_clone_const", |b| {
        b.iter(|| {
            let mut my_arr = black_box(MY_VEC);
            my_arr.push(4);
        })
    });
}

fn normal_push(c: &mut Criterion) {
    c.bench_function("normal_push", |b| {
        b.iter(|| {
            let mut my_arr = black_box(Vec::new());
            my_arr.push(4);
        })
    });
}

fn normal_push_precap(c: &mut Criterion) {
    c.bench_function("normal_push_precap", |b| {
        b.iter(|| {
            let mut my_arr = black_box(Vec::with_capacity(1));
            my_arr.push(4);
        })
    });
}

fn get(c: &mut Criterion) {
    c.bench_function("get", |b| {
        b.iter(|| {
            let my_arr = combo_vec![1, 2, 3];
            black_box(my_arr.get(1));
        })
    });
}

fn get_panic(c: &mut Criterion) {
    c.bench_function("get_panic", |b| {
        b.iter(|| {
            let my_arr = combo_vec![1, 2, 3];
            black_box(my_arr[1]);
        })
    });
}

criterion_group!(gets, get, get_panic);
criterion_group!(news, new, new_from_arr);
criterion_group!(
    pushes,
    normal_push,
    normal_push_precap,
    push,
    push_clone_const,
    push_no_vec,
    push_clone_const_no_vec
);
criterion_main!(gets, news, pushes);
