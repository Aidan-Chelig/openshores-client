use super::*;

#[test]
#[cfg(feature = "client")]
fn offline_native_preview_has_no_worker_or_network_request_channel() {
    let runtime = ClientRuntime::offline();
    assert!(runtime.worker.is_none());
    assert!(runtime.requests.is_none());
    assert!(runtime.events.is_none());
    assert!(runtime.send(Request::Disconnect).is_err());
    assert!(matches!(
        runtime.next_event(),
        Err(TryRecvError::Disconnected)
    ));
}

#[test]
fn unoffered_story_reply_is_rejected_without_network_io() {
    let runtime = ClientRuntime::start(LocalEndpoint {
        login: "127.0.0.1:1".parse().unwrap(),
        scene_port: 1,
        chat_port: 1,
    })
    .unwrap();
    runtime
        .send(Request::Choice(crate::chat::Reply {
            instance: 1,
            index: 0,
            offered: 700,
        }))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match runtime.next_event() {
            Ok(Event::Error(message)) => {
                assert!(message.contains("not offered"));
                break;
            }
            Err(TryRecvError::Empty) => {
                assert!(Instant::now() < deadline);
                thread::yield_now();
            }
            _ => panic!("unexpected event"),
        }
    }
}
#[test]
fn external_and_zero_endpoints_fail_before_worker_creation() {
    for endpoint in [
        LocalEndpoint {
            login: "192.0.2.1:16757".parse().unwrap(),
            scene_port: 16759,
            chat_port: 16758,
        },
        LocalEndpoint {
            login: "127.0.0.1:0".parse().unwrap(),
            scene_port: 16759,
            chat_port: 16758,
        },
    ] {
        assert!(ClientRuntime::start(endpoint).is_err());
    }
}
#[test]
fn unauthenticated_join_is_rejected_without_scene_connect() {
    let runtime = ClientRuntime::start(LocalEndpoint {
        login: "127.0.0.1:1".parse().unwrap(),
        scene_port: 1,
        chat_port: 1,
    })
    .unwrap();
    runtime.send(Request::Join(42)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match runtime.next_event() {
            Ok(Event::Error(message)) => {
                assert!(message.contains("authenticated roster"));
                break;
            }
            Err(TryRecvError::Empty) => {
                assert!(Instant::now() < deadline);
                thread::yield_now();
            }
            _ => panic!("unexpected event"),
        }
    }
}
