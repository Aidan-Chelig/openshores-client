use super::panels::{button, label};
use super::*;

pub(super) fn panel(commands: &mut Commands, root: Entity, ui: &Presentation) {
    if ui.model.story_open {
        for (control, title) in [
            (crate::chat::StoryControl::Close, "Close story"),
            (crate::chat::StoryControl::Hint, "Request hint"),
            (crate::chat::StoryControl::Resend, "Resend current block"),
        ] {
            button(commands, root, Action::StoryControl(control), title);
        }
    }
    for story in ui
        .model
        .story
        .iter()
        .rev()
        .take(8)
        .rev()
        .filter(|_| ui.model.story_open)
    {
        let title = match &story.header.channel {
            crate::chat::Channel::Named(name) => name.as_deref().unwrap_or("Story"),
            _ => story
                .header
                .sender_name
                .as_deref()
                .filter(|name| !name.is_empty())
                .unwrap_or("Story"),
        };
        label(
            commands,
            root,
            format!("[{title}] {}", story.header.text.as_deref().unwrap_or("")),
        );
    }
    button(commands, root, Action::Hail, "Hail nearby native (Voice)");
    for choice in &ui.model.choices {
        button(
            commands,
            root,
            Action::Reply(choice.reply()),
            format!(
                "{}: {}",
                choice
                    .header
                    .sender_name
                    .as_deref()
                    .filter(|name| !name.is_empty())
                    .unwrap_or(if choice.header.sender == 0 {
                        "Narrator"
                    } else {
                        "Speaker"
                    }),
                choice.prompt.as_deref().unwrap_or("Reply")
            ),
        );
    }
    button(
        commands,
        root,
        Action::NextChannel,
        format!(
            "Chat channel: {}",
            crate::chat::CHANNELS[ui.chat_channel as usize]
        ),
    );
    button(
        commands,
        root,
        Action::NextRange,
        format!("Chat range: {}", ui.chat_range.name()),
    );
    button(
        commands,
        root,
        Action::ChatFocus,
        format!("Message: {}", ui.chat_draft),
    );
    button(commands, root, Action::SendMessage, "Send message");
    if ui.chat_focus {
        label(
            commands,
            root,
            "Type message • Enter sends • Esc stops typing",
        );
    }
    if !ui.model.chat.is_empty() {
        label(commands, root, "Chat • received messages");
        for message in ui.model.chat.iter().rev().take(8).rev() {
            let channel = match &message.channel {
                crate::chat::Channel::Indexed(index) => crate::chat::CHANNELS
                    .get(*index as usize)
                    .copied()
                    .unwrap_or("Unknown"),
                crate::chat::Channel::Named(name) => name.as_deref().unwrap_or("Unnamed"),
            };
            label(
                commands,
                root,
                format!(
                    "[{channel}] {}: {}",
                    message.sender_name.as_deref().unwrap_or("Unknown"),
                    message.text.as_deref().unwrap_or("")
                ),
            );
        }
    }
}

pub(super) fn typing(
    mut ui: ResMut<Presentation>,
    bridge: Res<Bridge>,
    mut typed: MessageReader<KeyboardInput>,
) {
    if !ui.chat_focus || ui.model.phase != Phase::Playing {
        typed.clear();
        return;
    }
    for event in typed
        .read()
        .filter(|event| event.state == ButtonState::Pressed)
    {
        if !ui.chat_focus {
            break;
        }
        match event.key_code {
            KeyCode::Escape => ui.chat_focus = false,
            KeyCode::Enter => activate(Action::SendMessage, &mut ui, &bridge),
            KeyCode::Backspace => {
                ui.chat_draft.pop();
            }
            _ => {
                if let Some(text) = &event.text {
                    for character in text.chars().filter(|c| !c.is_control()) {
                        if ui.chat_draft.chars().count() < 256 {
                            ui.chat_draft.push(character);
                        }
                    }
                }
            }
        }
        ui.dirty = true;
    }
}
