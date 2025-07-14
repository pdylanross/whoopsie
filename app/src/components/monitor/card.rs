use crate::components::util::combine_optional_class;
use crate::types::{get_monitor, Monitor, MonitorStatus};
use leptos::leptos_dom::warn;
use leptos::prelude::*;
use std::time::Duration;

#[component]
pub fn MonitorCard(
    #[prop(optional)] class: Option<&'static str>,
    monitor: Monitor,
) -> impl IntoView {
    let monitor_id = monitor.name.clone();
    let (monitor, set_monitor) = signal(monitor);

    // Create a stable reference to set_monitor
    let update_monitor = Action::new(move |_: &()| {
        let monitor_id = monitor_id.clone();
        async move {
            match get_monitor(monitor_id).await {
                Ok(new_monitor) => set_monitor.set(new_monitor),
                Err(err) => {
                    warn!("Failed to refresh monitor: {}", err);
                }
            }
        }
    });

    // Set up the interval using create_effect
    let effector = Effect::new(move |_| {
        let _ = set_interval_with_handle(
            move || {
                update_monitor.dispatch(());
            },
            Duration::from_secs(15),
        );
    });

    on_cleanup(move || {
        effector.stop();
    });

    view! {
        <div class=combine_optional_class("card", class)>
            <div class="card-header grid grid-flow-col grid-cols-5 w-full">
                <div class="col-span-4">{move || monitor.get().name}</div>
                <div class=""><MonitorStatusLight status_fn={move || monitor().current_status} /></div>
            </div>
            <div><MonitorDetail monitor=move || monitor().current_status /></div>
        </div>
    }
}

#[component]
pub fn MonitorDetail(
    monitor: impl Fn() -> Option<MonitorStatus> + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div>{move || match monitor() {
            Some(st) => {
                match st {
                    MonitorStatus::Up { .. } => "up".into(),
                    MonitorStatus::Down { error_reason,.. } => format!("down: {error_reason}"),
                    MonitorStatus::Unknown => "unknown".into(),
                }
            },
            None => "unknown".into(),
        }}</div>
    }
}

#[component]
pub fn MonitorStatusLight(status_fn: impl Fn() -> Option<MonitorStatus>) -> impl IntoView {
    let base_classes = "justify-center text-center align-middle text-xs w-full h-full rounded-full";
    let mut background = "bg-gray-500";
    let text_color = "text-white";
    let mut status_message = "Unknown";

    match status_fn() {
        None => {}
        Some(status) => match status {
            MonitorStatus::Up { .. } => {
                background = "bg-green-500";
                status_message = "Up";
            }
            MonitorStatus::Down { .. } => {
                background = "bg-red-500";
                status_message = "Down";
            }
            MonitorStatus::Unknown => {}
        },
    }

    let classes = format!("{base_classes} {background} {text_color}");

    view! {
        <div class=classes>{status_message}</div>
    }
}
