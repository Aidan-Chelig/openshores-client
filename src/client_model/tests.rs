use super::*;
fn narration(target: u32, instance: i32) -> crate::chat::StoryText {
    crate::chat::StoryText {
        kind: 0x4a,
        instance,
        header: crate::chat::Message {
            channel: crate::chat::Channel::Named(Some("Getting Started".into())),
            flags: 15,
            sender: 0,
            sender_name: Some(String::new()),
            target,
            range: None,
            scope: 10,
            auxiliary: None,
            text: Some("Instruction".into()),
        },
    }
}
#[test]
fn story_inbox_is_bounded_and_narrated_choices_belong_to_selected_avatar() {
    let mut model = ClientModel {
        avatar: Some(512),
        ..Default::default()
    };
    model
        .apply(Event::IncomingChat(crate::chat::Incoming::Story(
            narration(513, 1),
        )))
        .unwrap();
    assert!(model.story.is_empty());
    for instance in 0..40 {
        model
            .apply(Event::IncomingChat(crate::chat::Incoming::Story(
                narration(512, instance),
            )))
            .unwrap();
    }
    assert_eq!(model.story.len(), 32);
    assert_eq!(model.story.front().unwrap().instance, 8);
    let choice = crate::chat::Choice {
        narrated: true,
        header: narration(512, 4097).header,
        instance: 4097,
        index: 2,
        offered_to: 512,
        prompt: Some("Go on".into()),
    };
    let reply = choice.reply();
    assert_eq!(reply.offered, 0);
    model
        .apply(Event::IncomingChat(crate::chat::Incoming::Choice(choice)))
        .unwrap();
    assert_eq!(model.choices.len(), 1);
    model.apply(Event::ChoiceSent(reply)).unwrap();
    assert!(model.choices.is_empty());
    assert_eq!(model.story.len(), 32); // No fabricated next instruction.
    assert!(model.story_open);
    model
        .apply(Event::StoryControlSent(crate::chat::StoryControl::Hint))
        .unwrap();
    assert!(model.story_open);
    model
        .apply(Event::StoryControlSent(crate::chat::StoryControl::Close))
        .unwrap();
    assert!(!model.story_open);
    assert_eq!(model.story.len(), 32); // Closing a panel does not delete history.
}
#[test]
fn local_reports_do_not_mutate_received_entity_transforms() {
    let mut model = ClientModel::default();
    let report = SceneCommand::PositionReport {
        projected: [1., 2., 3.],
        rotation: [4., 5., 6.],
    };
    model.apply(Event::SentScene(report)).unwrap();
    assert!(model.world.entities.is_empty());
    assert_eq!(model.position(), Some([1., 2., 3.]));
    assert!(model.status.contains("not an acknowledgement"));
}
