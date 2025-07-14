use crate::components::monitor::MonitorCard;
use crate::components::util::combine_optional_class;
use crate::types::Monitor;
use leptos::prelude::*;

#[component]
pub fn MonitorList(
    #[prop(optional)] class: Option<&'static str>,
    monitors: Vec<Monitor>,
) -> impl IntoView {
    let monitor_views = monitors
        .into_iter()
        .map(|m| {
            view! {
                <MonitorCard monitor={m} />
            }
        })
        .collect_view();

    view! {
        <div class=combine_optional_class("grid grid-flow-row grid-cols-1 space-y-4 space-x-4", class)>
            {monitor_views}
        </div>
    }
}
