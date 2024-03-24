use arrayvec::ArrayVec;
use combo_vec::{combo_vec, re_arr, ComboVec, ReArr};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smallvec::SmallVec;

const MY_VEC: ComboVec<i32, 8> = combo_vec![];
const MY_ARR: ReArr<i32, 8> = re_arr![];
const SMALL_VEC: SmallVec<[i32; 8]> = SmallVec::new_const();

fn new(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(|| black_box(ComboVec::<i32, 3>::new())));
}

fn new_from_arr(c: &mut Criterion) {
    c.bench_function("new_macro", |b| b.iter(|| black_box(combo_vec![1, 2, 3])));
}

fn push(c: &mut Criterion) {
    c.bench_function("push", |b| {
        b.iter(|| {
            let mut my_arr = black_box(ComboVec::<i32, 8>::new());
            my_arr.push(4);
        });
    });
}

fn push_no_vec(c: &mut Criterion) {
    c.bench_function("push_no_vec", |b| {
        b.iter(|| {
            let mut my_arr = black_box(ReArr::<i32, 8>::new());
            my_arr.push(4);
        });
    });
}

fn push_clone_const_no_vec(c: &mut Criterion) {
    c.bench_function("push_clone_const_no_vec", |b| {
        b.iter(|| {
            let mut my_arr = black_box(MY_ARR);
            my_arr.push(4);
        });
    });
}

fn push_clone_const(c: &mut Criterion) {
    c.bench_function("push_clone_const", |b| {
        b.iter(|| {
            let mut my_arr = black_box(MY_VEC);
            my_arr.push(4);
        });
    });
}

fn push_big_combo(c: &mut Criterion) {
    c.bench_function("push_big_combo", |b| {
        b.iter(|| {
            const VEC: ComboVec<i32, 2048> = combo_vec![];
            let mut my_arr = VEC;
            for i in 0..2048 {
                black_box(&mut my_arr).push(black_box(i));
            }
        });
    });
}

fn push_big_arr(c: &mut Criterion) {
    c.bench_function("push_big_arr", |b| {
        b.iter(|| {
            const ARR: ReArr<i32, 2048> = re_arr![];
            let mut my_arr = ARR;
            for i in 0..2048 {
                black_box(&mut my_arr).push(black_box(i));
            }
        });
    });
}

fn normal_push(c: &mut Criterion) {
    c.bench_function("normal_push", |b| {
        b.iter(|| {
            let mut my_arr = black_box(Vec::new());
            my_arr.push(4);
        });
    });
}

fn normal_push_precap(c: &mut Criterion) {
    c.bench_function("normal_push_precap", |b| {
        b.iter(|| {
            let mut my_arr = black_box(Vec::with_capacity(1));
            my_arr.push(4);
        });
    });
}

fn get(c: &mut Criterion) {
    c.bench_function("get", |b| {
        b.iter(|| {
            let my_arr = combo_vec![1, 2, 3];
            black_box(my_arr.get(1));
        });
    });
}

fn get_panic(c: &mut Criterion) {
    c.bench_function("get_panic", |b| {
        b.iter(|| {
            let my_arr = combo_vec![1, 2, 3];
            black_box(my_arr[1]);
        });
    });
}

fn smallvec_push(c: &mut Criterion) {
    c.bench_function("smallvec_push", |b| {
        b.iter(|| {
            let mut my_arr = black_box(SmallVec::<[i32; 8]>::new());
            my_arr.push(4);
        });
    });
}

fn smallvec_push_big(c: &mut Criterion) {
    c.bench_function("smallvec_push_big", |b| {
        b.iter(|| {
            const SMALL_VEC: SmallVec<[i32; 2048]> = SmallVec::new_const();
            let mut my_arr = SMALL_VEC;
            for i in 0..2048 {
                black_box(&mut my_arr).push(black_box(i));
            }
        });
    });
}

fn smallvec_clone_const_push(c: &mut Criterion) {
    c.bench_function("smallvec_clone_const_push", |b| {
        b.iter(|| {
            let mut my_arr = black_box(SMALL_VEC);
            my_arr.push(4);
        });
    });
}

fn arrayvec_push_big(c: &mut Criterion) {
    c.bench_function("arrayvec_push_big", |b| {
        b.iter(|| {
            const ARRAYVEC: ArrayVec<i32, 2048> = ArrayVec::new_const();
            let mut my_arr = ARRAYVEC;
            for i in 0..2048 {
                black_box(&mut my_arr).push(black_box(i));
            }
        });
    });
}

fn vec_pop_big(c: &mut Criterion) {
    c.bench_function("vec_pop_big", |b| {
        b.iter(|| {
            const VEC: ComboVec<i32, 2048> = combo_vec![];
            let mut my_vec = VEC;
            for i in 0..2048 {
                black_box(&mut my_vec).push(black_box(i));
            }

            for _ in 0..2048 {
                black_box(&mut my_vec).pop();
            }
        });
    });
}

fn arr_pop_big(c: &mut Criterion) {
    c.bench_function("arr_pop_big", |b| {
        b.iter(|| {
            const ARR: ReArr<i32, 2048> = re_arr![];
            let mut my_arr = ARR;
            for i in 0..2048 {
                black_box(&mut my_arr).push(black_box(i));
            }

            for _ in 0..2048 {
                black_box(&mut my_arr).pop();
            }
        });
    });
}

fn smallvec_pop_big(c: &mut Criterion) {
    c.bench_function("smallvec_pop_big", |b| {
        b.iter(|| {
            const SMALL_VEC: SmallVec<[i32; 2048]> = SmallVec::new_const();
            let mut my_vec = SMALL_VEC;
            for i in 0..2048 {
                black_box(&mut my_vec).push(black_box(i));
            }

            for _ in 0..2048 {
                black_box(&mut my_vec).pop();
            }
        });
    });
}

fn arrayvec_pop_big(c: &mut Criterion) {
    c.bench_function("arrayvec_pop_big", |b| {
        b.iter(|| {
            const ARRAYVEC: ArrayVec<i32, 2048> = ArrayVec::new_const();
            let mut my_arr = ARRAYVEC;
            for i in 0..2048 {
                black_box(&mut my_arr).push(black_box(i));
            }

            for _ in 0..2048 {
                black_box(&mut my_arr).pop();
            }
        });
    });
}

criterion_group!(gets, get, get_panic);
criterion_group!(news, new, new_from_arr);
criterion_group!(
    pushes,
    push_big_combo,
    push_big_arr,
    normal_push,
    normal_push_precap,
    push,
    push_clone_const,
    push_no_vec,
    push_clone_const_no_vec,
);
criterion_group!(smallvec, smallvec_push, smallvec_clone_const_push, smallvec_push_big);
criterion_group!(arrayvec, arrayvec_push_big);
criterion_group!(pop, arr_pop_big, vec_pop_big, arrayvec_pop_big, smallvec_pop_big);
criterion_main!(pop, arrayvec, smallvec, gets, news, pushes);
