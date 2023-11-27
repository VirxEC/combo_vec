use combo_vec::{re_arr, ReArr};

const DEFAULT_TEST_REARR: ReArr<i32, 5> = re_arr![1, 2, 3; None, None];
const EMPTY_STRING_ALLOC: ReArr<String, 3> = re_arr![];

#[test]
#[cfg(feature = "alloc")]
fn copy_string_re_arr() {
    let mut x = EMPTY_STRING_ALLOC;
    x.push(String::from("hello"));
    x.push(String::from("world"));
    assert_eq!(x.join(" "), "hello world");
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
    assert_eq!(cv.capacity(), 5);
}

#[test]
fn extend() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.extend(vec![4]);
    assert_eq!(cv.len(), 4);
    #[cfg(feature = "alloc")]
    assert_eq!(cv.to_vec(), vec![1, 2, 3, 4]);
}

#[test]
fn truncate_push() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.truncate(2);
    cv.push(3);
    assert_eq!(cv.len(), 3);
    #[cfg(feature = "alloc")]
    assert_eq!(cv.to_vec(), vec![1, 2, 3]);
}

#[test]
fn truncate() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.truncate(2);
    assert_eq!(cv.len(), 2);
    #[cfg(feature = "alloc")]
    assert_eq!(cv.to_vec(), vec![1, 2]);
}

#[test]
fn truncate_invalids() {
    let mut cv = DEFAULT_TEST_REARR;
    cv.truncate(4);
    cv.truncate(3);
    assert_eq!(cv.len(), 3);
    #[cfg(feature = "alloc")]
    assert_eq!(cv.to_vec(), vec![1, 2, 3]);
}

#[test]
fn exarr_macro() {
    let item1 = re_arr![1, 2, 3];
    println!("{item1}");
    assert_eq!(item1.len(), 3);

    let item2 = re_arr![5; 3];
    println!("{item2}");
    assert_eq!(item2.len(), 3);
}
