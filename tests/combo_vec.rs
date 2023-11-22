#![cfg(feature = "alloc")]

use combo_vec::{combo_vec, ComboVec};

const DEFAULT_TEST_REARR: ComboVec<i32, 3> = combo_vec![1, 2, 3];
const EMPTY_STRING_ALLOC: ComboVec<String, 3> = combo_vec![];

#[test]
fn copy_string_combo_vec() {
    let mut x = EMPTY_STRING_ALLOC;
    x.push(String::from("hello"));
    x.push(String::from("world"));
    let final_sentance = vec![String::from("hello"), String::from("world")];
    assert_eq!(final_sentance.join(" "), "hello world");
}

#[test]
fn make_new() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.push(4);
    cv.push(5);
    println!("{cv}");
    dbg!(&cv);
    assert_eq!(cv.get(0), Some(&1));
    assert_eq!(cv.get(1), Some(&2));
    assert_eq!(cv.get(2), Some(&3));
    assert_eq!(cv.get(3), Some(&4));
    assert_eq!(cv.last(), Some(&5));
    assert_eq!(cv.get(4), Some(&5));
    assert_eq!(cv.get(5), None);
    assert_eq!(cv.get_mut(0), Some(&mut 1));
}

#[test]
fn iter() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.push(4);
    assert_eq!(cv.iter().collect::<Vec<_>>(), vec![&1, &2, &3, &4]);
    assert_eq!(cv.into_iter().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
}

#[test]
fn lengths() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.push(4);
    assert_eq!(cv.len(), 4);
    assert_eq!(cv.stack_len(), 3);
    assert_eq!(cv.heap_len(), 1);
}

#[test]
fn extend() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.extend(vec![4, 5, 6]);
    cv.extend(DEFAULT_TEST_REARR);
    dbg!(&cv);
    assert_eq!(cv.len(), 9);
    assert_eq!(cv.stack_len(), 3);
    assert_eq!(cv.heap_len(), 6);
    assert_eq!(cv.to_vec(), vec![1, 2, 3, 4, 5, 6, 1, 2, 3]);
}

#[test]
fn truncate_into_stack_push() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.truncate(2);
    cv.push(3);
    assert_eq!(cv.len(), 3);
    assert_eq!(cv.stack_len(), 3);
    assert_eq!(cv.heap_len(), 0);
    assert_eq!(cv.to_vec(), vec![1, 2, 3]);
}

#[test]
fn truncate_into_stack() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.truncate(2);
    assert_eq!(cv.len(), 2);
    assert_eq!(cv.stack_len(), 2);
    assert_eq!(cv.heap_len(), 0);
    assert_eq!(cv.to_vec(), vec![1, 2]);
}

#[test]
fn truncate_into_heap() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.extend(vec![4, 5, 6]);
    cv.truncate(4);
    assert_eq!(cv.len(), 4);
    assert_eq!(cv.stack_len(), 3);
    assert_eq!(cv.heap_len(), 1);
    assert_eq!(cv.to_vec(), vec![1, 2, 3, 4]);
}

#[test]
fn truncate_invalids() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.truncate(4);
    cv.truncate(3);
    assert_eq!(cv.len(), 3);
    assert_eq!(cv.stack_len(), 3);
    assert_eq!(cv.heap_len(), 0);
    assert_eq!(cv.to_vec(), vec![1, 2, 3]);
}

#[test]
fn exarr_macro() {
    let item1 = combo_vec![1, 2, 3];
    println!("{item1}");
    assert_eq!(item1.len(), 3);

    let item2 = combo_vec![5; 3];
    println!("{item2}");
    assert_eq!(item2.len(), 3);

    let item3 = combo_vec![i32];
    println!("{item3}");
    assert_eq!(item3.len(), 0);
    assert_eq!(item3.stack_capacity(), 16);

    let item4 = combo_vec![i32; 5];
    println!("{item4}");
    assert_eq!(item4.len(), 0);
    assert_eq!(item4.stack_capacity(), 5);
}
