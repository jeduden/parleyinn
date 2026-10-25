use crate::branch::tests::{branch, chain, link, room};

use super::*;

fn works(n: u8) -> RunSeatState {
    RunSeatState {
        room: room(n),
        member_with_work: true,
    }
}

fn idle(n: u8) -> RunSeatState {
    RunSeatState {
        room: room(n),
        member_with_work: false,
    }
}

#[test]
fn an_event_with_no_branch_goes_to_the_personal_room() {
    let links = [link(1, 1, 1, 1)];
    let target = seat_for_run_event(None, &[works(1)], &links, &chain);
    assert_eq!(target, SeatTarget::PersonalRoomSeat);
}

#[test]
fn no_rooms_and_no_links_go_to_the_personal_room() {
    let main = branch("main");
    let target = seat_for_run_event(Some(&main), &[], &[], &chain);
    assert_eq!(target, SeatTarget::PersonalRoomSeat);
}

#[test]
fn a_branch_no_link_names_goes_to_the_personal_room() {
    let links = [link(1, 1, 1, 1)];
    let other = branch("other");
    let target = seat_for_run_event(Some(&other), &[works(1)], &links, &chain);
    assert_eq!(target, SeatTarget::PersonalRoomSeat);
}

#[test]
fn a_member_with_work_in_the_linked_room_gets_the_event() {
    let links = [link(1, 1, 1, 1)];
    let main = branch("main");
    let target = seat_for_run_event(Some(&main), &[works(1)], &links, &chain);
    assert_eq!(target, SeatTarget::RunSeat(room(1)));
}

#[test]
fn a_seat_without_work_goes_to_the_personal_room_not_to_another_room() {
    // Room 1's link stands; the run's seat there is muted. Room 2 also
    // names the branch (void) and its seat works, but the branch is not
    // room 2's.
    let links = [link(1, 1, 1, 1), link(2, 2, 1, 2)];
    let main = branch("main");
    let rooms = [idle(1), works(2)];
    let target = seat_for_run_event(Some(&main), &rooms, &links, &chain);
    assert_eq!(target, SeatTarget::PersonalRoomSeat);
}

#[test]
fn two_rooms_naming_one_branch_route_to_the_standing_link() {
    let links = [link(2, 2, 1, 9), link(1, 1, 1, 1)];
    let main = branch("main");
    let rooms = [works(1), works(2)];
    let target = seat_for_run_event(Some(&main), &rooms, &links, &chain);
    assert_eq!(target, SeatTarget::RunSeat(room(1)));
}

#[test]
fn a_link_from_a_room_the_principal_lacks_routes_nowhere() {
    let links = [link(7, 7, 1, 0)];
    let main = branch("main");
    let target = seat_for_run_event(Some(&main), &[works(1)], &links, &chain);
    assert_eq!(target, SeatTarget::PersonalRoomSeat);
}

#[test]
fn a_foreign_room_link_does_not_void_the_principals_link() {
    // Room 7 is not among the principal's rooms and has the lower
    // commitment, but voids nothing.
    let links = [link(7, 7, 1, 0), link(1, 1, 1, 200)];
    let main = branch("main");
    let target = seat_for_run_event(Some(&main), &[works(1)], &links, &chain);
    assert_eq!(target, SeatTarget::RunSeat(room(1)));
}

#[test]
fn a_link_flagged_foreign_routes_nowhere() {
    let mut flagged = link(1, 1, 1, 1);
    flagged.foreign = true;
    let main = branch("main");
    let target = seat_for_run_event(Some(&main), &[works(1)], &[flagged], &chain);
    assert_eq!(target, SeatTarget::PersonalRoomSeat);
}
