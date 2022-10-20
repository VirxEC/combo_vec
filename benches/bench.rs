use combo_vec::{rearr, ReArr};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const MY_ARR: ReArr<i32, 8> = rearr![];

fn new(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(|| black_box(ReArr::<i32, 3>::new())));
}

fn new_from_arr(c: &mut Criterion) {
    c.bench_function("new_macro", |b| b.iter(|| black_box(rearr![1, 2, 3])));
}

fn push(c: &mut Criterion) {
    c.bench_function("push", |b| {
        b.iter(|| {
            let mut my_arr: ReArr<i32, 8> = black_box(rearr![]);
            my_arr.push(4);
        })
    });
}

fn push_clone_const(c: &mut Criterion) {
    c.bench_function("push_clone_const", |b| {
        b.iter(|| {
            let mut my_arr = black_box(MY_ARR);
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
            let my_arr = rearr![1, 2, 3];
            black_box(my_arr.get(1));
        })
    });
}

fn get_panic(c: &mut Criterion) {
    c.bench_function("get_panic", |b| {
        b.iter(|| {
            let my_arr = rearr![1, 2, 3];
            black_box(my_arr[1]);
        })
    });
}

criterion_group!(gets, get, get_panic);
criterion_group!(news, new, new_from_arr);
criterion_group!(pushes, normal_push, normal_push_precap, push, push_clone_const);
criterion_main!(/*gets, news, */ pushes);
