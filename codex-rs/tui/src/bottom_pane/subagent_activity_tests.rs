use super::*;
use pretty_assertions::assert_eq;

#[test]
fn completion_expires_without_repeated_updates_extending_the_deadline() {
    let now = Instant::now();
    let mut indicator = SubagentActivityIndicator::default();
    let running = SubagentActivity {
        running: 3,
        ..Default::default()
    };
    let done = SubagentActivity {
        done: 3,
        ..Default::default()
    };
    indicator.update(running, now);
    assert_eq!(
        indicator.visible_activity(now + Duration::from_secs(/*secs*/ 60)),
        Some(&running)
    );
    indicator.update(done, now);
    assert!(!indicator.update(done, now + Duration::from_secs(/*secs*/ 4)));
    assert_eq!(
        indicator.visible_activity(now + Duration::from_secs(/*secs*/ 4)),
        Some(&done)
    );
    assert_eq!(
        indicator.visible_activity(now + Duration::from_secs(/*secs*/ 5)),
        None
    );
    assert!(!indicator.update(done, now + Duration::from_secs(/*secs*/ 6)));
    assert_eq!(
        indicator.visible_activity(now + Duration::from_secs(/*secs*/ 6)),
        None
    );

    indicator.update(running, now + Duration::from_secs(/*secs*/ 7));
    assert_eq!(
        indicator.visible_activity(now + Duration::from_secs(/*secs*/ 60)),
        Some(&running)
    );
    indicator.update(done, now + Duration::from_secs(/*secs*/ 61));
    assert_eq!(
        indicator.visible_activity(now + Duration::from_secs(/*secs*/ 65)),
        Some(&done)
    );
    assert_eq!(
        indicator.visible_activity(now + Duration::from_secs(/*secs*/ 66)),
        None
    );
}

#[test]
fn unconfirmed_idle_agents_do_not_trigger_completion_timeout() {
    let now = Instant::now();
    let mut indicator = SubagentActivityIndicator::default();
    let activity = SubagentActivity {
        done: 2,
        idle: 1,
        ..Default::default()
    };
    indicator.update(activity, now);
    assert_eq!(
        indicator.visible_activity(now + Duration::from_secs(/*secs*/ 60)),
        Some(&activity)
    );
    indicator.update(SubagentActivity::default(), now);
    assert_eq!(indicator.visible_activity(now), None);
}

#[test]
fn completion_timeout_and_reactivation_snapshot() {
    let now = Instant::now();
    let mut indicator = SubagentActivityIndicator::default();
    indicator.update(
        SubagentActivity {
            done: 3,
            ..Default::default()
        },
        now,
    );
    let mut stages = Vec::new();
    for seconds in [0, 4, 5] {
        let line = indicator
            .visible_activity(now + Duration::from_secs(seconds))
            .map(|activity| activity.line(/*width*/ 80).to_string())
            .unwrap_or_default();
        stages.push(format!("{seconds}s:{line}"));
    }
    indicator.update(
        SubagentActivity {
            running: 1,
            done: 2,
            idle: 0,
        },
        now + Duration::from_secs(/*secs*/ 6),
    );
    let line = indicator
        .visible_activity(now + Duration::from_secs(/*secs*/ 6))
        .expect("new activity is visible")
        .line(/*width*/ 80);
    stages.push(format!("6s:{line}"));
    insta::assert_snapshot!(stages.join("\n"), @r"
    0s:  ○ Agents  3 done  /subagents
    4s:  ○ Agents  3 done  /subagents
    5s:
    6s:  ● Agents  1 running · 2 done  /subagents
    ");
}

#[test]
fn subagent_activity_adapts_to_width_and_completion() {
    let mut activity = SubagentActivity {
        running: 3,
        done: 1,
        idle: 0,
    };
    let mut snapshots = Vec::new();
    for width in [80, 40, 20, 8] {
        let area = Rect::new(
            /*x*/ 0,
            /*y*/ 0,
            width,
            activity.desired_height(width),
        );
        let mut buffer = Buffer::empty(area);
        activity.render(area, &mut buffer);
        let row: String = (0..width).map(|x| buffer[(x, 0)].symbol()).collect();
        snapshots.push(format!("{width}: {}", row.trim_end()));
    }
    activity.running = 0;
    activity.done = 4;
    snapshots.push(activity.line(/*width*/ 80).to_string());
    activity.idle = 1;
    snapshots.push(activity.line(/*width*/ 80).to_string());
    insta::assert_snapshot!(snapshots.join("\n"));
    assert_eq!(activity.desired_height(/*width*/ 80), 1);
}

#[test]
fn subagent_activity_styles_only_active_work_with_an_accent() {
    let active = SubagentActivity {
        running: 1,
        ..Default::default()
    };
    let area = Rect::new(
        /*x*/ 0, /*y*/ 0, /*width*/ 60, /*height*/ 1,
    );
    let mut buffer = Buffer::empty(area);
    active.render(area, &mut buffer);
    assert_eq!(buffer[(2, 0)].fg, ratatui::style::Color::Cyan);
    let finished = SubagentActivity {
        done: 1,
        ..Default::default()
    };
    let mut buffer = Buffer::empty(area);
    finished.render(area, &mut buffer);
    assert!(
        buffer[(2, 0)]
            .modifier
            .contains(ratatui::style::Modifier::DIM)
    );
}
