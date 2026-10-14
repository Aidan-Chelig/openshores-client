use super::*;
#[test]
fn modes_preserve_distinct_branches_signatures_and_shared_tail() {
    assert!(connection_plan(9).is_empty());
    for (mode, count) in [(5, 10), (10, 57), (0, 52), (-1, 52), (i32::MAX, 52)] {
        let plan = connection_plan(mode);
        assert_eq!(plan.len(), count);
        assert_eq!(plan[0].raw_type, 0x82);
        if mode == 5 {
            assert_eq!(plan.last().unwrap().signal, "2CreateEmpire()");
            assert!(!plan.iter().any(|b| b.signal.starts_with("2GameTurnPoll")));
        } else {
            assert!(plan.last().unwrap().signal.starts_with("2WormholeExplored"));
            let turn = plan
                .iter()
                .find(|b| b.signal.starts_with("2GameTurnPoll"))
                .unwrap();
            assert_eq!(
                turn.target.as_bytes()[0],
                if mode == 10 { b'1' } else { b'2' }
            );
            assert_eq!(plan.iter().filter(|b| b.raw_type == 0).count(), 2);
        }
    }
}
struct Connector {
    calls: Vec<NativeSignalConnection>,
    reject_first: bool,
}
impl NativeSignalConnector for Connector {
    fn connect(&mut self, handle: u32, binding: NativeSignalConnection) -> io::Result<bool> {
        assert_eq!(handle, 90);
        self.calls.push(binding);
        Ok(!(self.reject_first || self.calls.len() == 2))
    }
}
#[test]
fn first_rejection_stops_but_later_rejection_does_not() {
    for mode in [5, 10, 0] {
        let mut connector = Connector {
            calls: vec![],
            reject_first: true,
        };
        assert_eq!(
            connect_signals(90, mode, &mut connector).unwrap().attempted,
            1
        );
        connector.reject_first = false;
        connector.calls.clear();
        let result = connect_signals(90, mode, &mut connector).unwrap();
        assert_eq!(connector.calls, connection_plan(mode));
        assert_eq!(result.accepted + 1, result.attempted);
        assert!(!result.first_rejected);
    }
    let mut connector = Connector {
        calls: vec![],
        reject_first: true,
    };
    assert_eq!(connect_signals(90, 9, &mut connector).unwrap().attempted, 0);
    assert!(connector.calls.is_empty());
}
