use std::fs;

use super::*;
use crate::home::Home;
use crate::node::Node;
use crate::node_identity::IdentitySource;
use key::random::OsEntropy;
use key::signing::SeatKey;
use record::range::Label;
use testkit::TempDir;

fn home(dir: &TempDir) -> Home {
    fs::write(dir.path().join("machine-id"), "first\n").unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    home
}

fn run_seat(node: &Node, store: &mut Store) -> WriterId {
    let key = SeatKey::mint(&mut OsEntropy).unwrap();

    node.certify_run_seat(store, &key, &mut OsEntropy)
        .unwrap()
        .writer
}

#[test]
fn labels_go_to_run_seats_in_the_order_they_were_certified_then_to_the_device_seat() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let first = run_seat(&node, &mut store);
    let second = run_seat(&node, &mut store);

    let assigned = assign_missing(&mut store, node.device_writer(), &mut OsEntropy).unwrap();

    assert_eq!(assigned, 3);
    let labels = store.writer_labels(node.device_writer()).unwrap();
    let label = |n| Label::new(n).unwrap();
    assert_eq!(labels[&first], label(1));
    assert_eq!(labels[&second], label(2));
    assert_eq!(labels[&node.device_writer()], label(3));
}

#[test]
fn a_later_seat_takes_the_next_label_and_a_labelled_node_assigns_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let first = run_seat(&node, &mut store);
    assign_missing(&mut store, node.device_writer(), &mut OsEntropy).unwrap();
    let before = store.log(node.device_writer()).unwrap().len();
    assert_eq!(
        assign_missing(&mut store, node.device_writer(), &mut OsEntropy).unwrap(),
        0
    );
    assert_eq!(store.log(node.device_writer()).unwrap().len(), before);

    let later = run_seat(&node, &mut store);
    assert_eq!(
        assign_missing(&mut store, node.device_writer(), &mut OsEntropy).unwrap(),
        1
    );

    let labels = store.writer_labels(node.device_writer()).unwrap();
    assert_eq!(labels[&first], Label::new(1).unwrap());
    assert_eq!(labels[&later], Label::new(3).unwrap());
    let unique: BTreeSet<_> = labels.values().collect();
    assert_eq!(unique.len(), labels.len());
}

#[test]
fn an_assignment_is_a_structural_event_of_the_device_seat() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let seat = run_seat(&node, &mut store);

    assign_missing(&mut store, node.device_writer(), &mut OsEntropy).unwrap();

    let log = store.log(node.device_writer()).unwrap();
    let assignments: Vec<_> = log
        .iter()
        .filter(|s| s.link.header.kind == EventKind::WriterLabel)
        .collect();
    assert_eq!(assignments.len(), 2);
    assert!(
        assignments
            .iter()
            .all(|s| s.link.header.provenance == Provenance::Structural)
    );
    assert_eq!(
        assignments[0].content.as_ref().unwrap()["writer"],
        seat.to_string()
    );
}
