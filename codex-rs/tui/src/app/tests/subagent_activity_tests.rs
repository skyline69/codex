use super::*;
use crate::bottom_pane::SubagentActivity;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn subagent_activity_tracks_background_turns_and_session_reset() -> Result<()> {
    let mut app = make_test_app().await;
    let root = ThreadId::new();
    let child = ThreadId::new();
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    app.upsert_agent_picker_thread(
        root, /*agent_nickname*/ None, /*agent_role*/ None, /*is_closed*/ false,
    );
    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: root.to_string(),
            turn_id: "parent-turn".into(),
            started_at_ms: 0,
            item: ThreadItem::SubAgentActivity {
                id: "spawn-child".into(),
                kind: codex_app_server_protocol::SubAgentActivityKind::Started,
                agent_thread_id: child.to_string(),
                agent_path: "/root/child".into(),
            },
        }),
    )));
    insta::assert_snapshot!(
        "subagent_activity_parent_idle",
        render_bottom_popup(&app.chat_widget, /*width*/ 80)
    );

    app.enqueue_thread_notification(child, turn_started_notification(child, "child-turn"))
        .await?;
    app.enqueue_thread_notification(
        child,
        turn_completed_notification(child, "child-turn", TurnStatus::Completed),
    )
    .await?;
    assert!(render_bottom_popup(&app.chat_widget, /*width*/ 80).contains("○ Agents  1 done"));

    // A late spawn hint cannot revive an agent whose turn already finished.
    app.agent_navigation
        .record_sub_agent_activity(SubAgentActivityDisplay {
            thread_id: child,
            agent_path: "/root/child".into(),
            is_running_hint: true,
        });
    app.active_thread_id = Some(child);
    app.sync_active_agent_label();
    assert!(render_bottom_popup(&app.chat_widget, /*width*/ 80).contains("○ Agents  1 done"));
    app.enqueue_thread_notification(child, turn_started_notification(child, "child-turn-2"))
        .await?;
    assert!(render_bottom_popup(&app.chat_widget, /*width*/ 80).contains("● Agents  1 running"));

    app.reset_thread_event_state();
    assert!(!render_bottom_popup(&app.chat_widget, /*width*/ 80).contains("Agents  "));
    Ok(())
}

#[test]
fn subagent_activity_does_not_treat_unknown_threads_as_completed() {
    let mut state = AgentNavigationState::default();
    let root = ThreadId::new();
    let child = ThreadId::new();
    let side = ThreadId::new();
    for thread in [root, child, side] {
        state.upsert(
            thread, /*agent_nickname*/ None, /*agent_role*/ None,
            /*is_closed*/ false,
        );
    }
    let included = |thread| thread != root && thread != side;
    assert_eq!(
        state.activity_summary(included),
        SubagentActivity {
            idle: 1,
            ..Default::default()
        }
    );
    state.mark_running(child);
    assert_eq!(
        state.activity_summary(included),
        SubagentActivity {
            running: 1,
            ..Default::default()
        }
    );
    state.mark_closed(child);
    assert_eq!(
        state.activity_summary(included),
        SubagentActivity {
            done: 1,
            ..Default::default()
        }
    );
    state.remove(child);
    assert_eq!(
        state.activity_summary(included),
        SubagentActivity::default()
    );
}
