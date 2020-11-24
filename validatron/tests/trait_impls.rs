use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, LinkedList, VecDeque};
use validatron::{Error, Location, Result, Validate};

#[derive(PartialOrd, Ord, PartialEq, Eq)]
struct Dummy(bool);

impl Validate for Dummy {
    fn validate(&self) -> Result<()> {
        if self.0 {
            Ok(())
        } else {
            Err(Error::new("false"))
        }
    }
}
impl<'a> Validate for &'a Dummy {
    fn validate(&self) -> Result<()> {
        (*self).validate()
    }
}

#[test]
fn option() {
    let a: Option<Dummy> = None;
    assert!(a.validate().is_ok());

    assert_eq!(Validate::validate(&a).is_ok(), true);
    assert_eq!(a.validate().is_ok(), true);

    let b = Some(Dummy(true));
    assert_eq!(b.validate().is_ok(), true);

    let b = Some(Dummy(false));
    assert_eq!(b.validate().is_ok(), false);
}

#[test]
fn vec() {
    let mut inp: Vec<Dummy> = vec![];
    assert!(inp.validate().is_ok());

    inp = vec![Dummy(true)];
    assert!(inp.validate().is_ok());

    inp.push(Dummy(false));
    assert!(inp.validate().is_err());

    inp.push(Dummy(true));
    inp.push(Dummy(false));

    let e = inp.validate().unwrap_err();
    match e {
        Error::Structured(map) => {
            assert_eq!(map.len(), 2);

            assert_eq!(map.contains_key(&Location::Index(0)), false);
            assert_eq!(map.contains_key(&Location::Index(1)), true);
            assert_eq!(map.contains_key(&Location::Index(2)), false);
            assert_eq!(map.contains_key(&Location::Index(3)), true);
        }
        _ => panic!("cannot happen"),
    }
}

#[test]
fn vecdeque() {
    let mut inp: VecDeque<Dummy> = VecDeque::new();
    assert!(inp.validate().is_ok());

    inp.push_back(Dummy(true));
    assert!(inp.validate().is_ok());

    inp.push_back(Dummy(false));
    assert!(inp.validate().is_err());

    inp.push_back(Dummy(true));
    inp.push_back(Dummy(false));

    let e = inp.validate().unwrap_err();
    match e {
        Error::Structured(map) => {
            assert_eq!(map.len(), 2);

            assert_eq!(map.contains_key(&Location::Index(0)), false);
            assert_eq!(map.contains_key(&Location::Index(1)), true);
            assert_eq!(map.contains_key(&Location::Index(2)), false);
            assert_eq!(map.contains_key(&Location::Index(3)), true);
        }
        _ => panic!("cannot happen"),
    }
}

#[test]
fn linkedlist() {
    let mut inp: LinkedList<Dummy> = LinkedList::new();
    assert!(inp.validate().is_ok());

    inp.push_back(Dummy(true));
    assert!(inp.validate().is_ok());

    inp.push_back(Dummy(false));
    assert!(inp.validate().is_err());

    inp.push_back(Dummy(true));
    inp.push_back(Dummy(false));

    let e = inp.validate().unwrap_err();
    match e {
        Error::Structured(map) => {
            assert_eq!(map.len(), 2);

            assert_eq!(map.contains_key(&Location::Index(0)), false);
            assert_eq!(map.contains_key(&Location::Index(1)), true);
            assert_eq!(map.contains_key(&Location::Index(2)), false);
            assert_eq!(map.contains_key(&Location::Index(3)), true);
        }
        _ => panic!("cannot happen"),
    }
}

#[test]
fn hashmap() {
    let mut data = HashMap::new();

    assert!(data.validate().is_ok());

    data.insert("a place", Dummy(true));
    assert!(data.validate().is_ok());

    data.insert("a different place", Dummy(false));
    assert!(data.validate().is_err());

    let e = data.validate().unwrap_err();
    match e {
        Error::Field(_) => panic!("should happen"),
        Error::Structured(x) => {
            assert_eq!(x.len(), 1);
            assert!(x.contains_key(&Location::MapKey("a different place".into())));
        }
    }
}

#[test]
fn btreemap() {
    let mut data = BTreeMap::new();

    assert!(data.validate().is_ok());

    data.insert("a place", Dummy(true));
    assert!(data.validate().is_ok());

    data.insert("a different place", Dummy(false));
    assert!(data.validate().is_err());

    let e = data.validate().unwrap_err();
    match e {
        Error::Field(_) => panic!("should happen"),
        Error::Structured(x) => {
            assert_eq!(x.len(), 1);
            assert!(x.contains_key(&Location::MapKey("a different place".into())));
        }
    }
}

#[test]
fn btreeset() {
    let mut data = BTreeSet::new();
    assert!(data.validate().is_ok());

    data.insert(Dummy(true));
    assert!(data.validate().is_ok());

    data.insert(Dummy(false));
    assert!(data.validate().is_err());

    let e = data.validate().unwrap_err();
    println!("{:#?}", e);
    match e {
        Error::Field(_) => panic!("should happen"),
        Error::Structured(x) => {
            assert_eq!(x.len(), 1);
            assert!(x.contains_key(&Location::Index(0)));
        }
    }
}

#[test]
fn binaryheap() {
    let mut data = BinaryHeap::new();
    assert!(data.validate().is_ok());

    data.push(Dummy(true));
    assert!(data.validate().is_ok());

    data.push(Dummy(false));
    assert!(data.validate().is_err());

    let e = data.validate().unwrap_err();
    println!("{:#?}", e);
    match e {
        Error::Field(_) => panic!("should happen"),
        Error::Structured(x) => {
            assert_eq!(x.len(), 1);
            assert!(x.contains_key(&Location::Index(1)));
        }
    }
}
