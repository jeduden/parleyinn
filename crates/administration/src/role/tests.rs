use super::*;
use key::random::OsEntropy;
use key::signing::{DeviceKey, SeatKey};
use record::id::RoomId;

fn cert(device: &DeviceKey, kind: SeatKind, ingest: bool) -> SeatCertificate {
    let seat = SeatKey::mint(&mut OsEntropy).unwrap();
    let room = RoomId::derive(&[1; 32], &[2; 16]);
    SeatCertificate::issue(device, &seat, room, kind, ingest).unwrap()
}

#[test]
fn each_role_carries_exactly_its_capabilities() {
    use Capability::*;
    assert_eq!(
        Role::Viewer.capabilities(),
        [Read, RoleRequest, SummaryRequest].into_iter().collect()
    );
    assert_eq!(
        Role::Contributor.capabilities(),
        [
            Read,
            SummaryRequest,
            Post,
            Link,
            Present,
            Pin,
            Edit,
            Unpin,
            Work
        ]
        .into_iter()
        .collect()
    );
    let moderator = Role::Moderator.capabilities();
    assert!(Role::Contributor.capabilities().is_subset(&moderator));
    let extra: BTreeSet<Capability> = moderator
        .difference(&Role::Contributor.capabilities())
        .copied()
        .collect();
    assert_eq!(
        extra,
        [
            ListRemoval,
            Kick,
            Bar,
            Unbar,
            Mute,
            Unmute,
            Pick,
            SetTitle,
            SetLabels,
            SetAssignments
        ]
        .into_iter()
        .collect()
    );
    assert!(!moderator.contains(&RoomSummary));
    assert!(!moderator.contains(&RoleRequest));
}

#[test]
fn the_owners_device_seat_has_every_capability_but_writing_a_room_summary() {
    let device = DeviceKey::mint(&mut OsEntropy).unwrap();
    let own = [device.public()];
    let seat = cert(&device, SeatKind::Device, false);

    assert_eq!(seat_role(&seat, &own), None);
    let held = seat_capabilities(&seat, &own);
    assert_eq!(held.len(), CAPABILITIES.len() - 1);
    assert!(!held.contains(&Capability::RoomSummary));
}

#[test]
fn a_runs_seat_and_the_seat_ingest_starts_are_contributors() {
    let device = DeviceKey::mint(&mut OsEntropy).unwrap();
    let own = [device.public()];

    for ingest in [false, true] {
        let seat = cert(&device, SeatKind::Run, ingest);
        assert_eq!(seat_role(&seat, &own), Some(Role::Contributor));
        assert_eq!(
            seat_capabilities(&seat, &own),
            Role::Contributor.capabilities()
        );
    }
}

#[test]
fn a_seat_no_key_of_this_node_certified_is_a_viewer() {
    let device = DeviceKey::mint(&mut OsEntropy).unwrap();
    let other = DeviceKey::mint(&mut OsEntropy).unwrap();
    let own = [device.public()];

    for kind in [SeatKind::Device, SeatKind::Run] {
        let seat = cert(&other, kind, false);
        assert_eq!(seat_role(&seat, &own), Some(Role::Viewer));
        assert_eq!(seat_capabilities(&seat, &own), Role::Viewer.capabilities());
    }
}
