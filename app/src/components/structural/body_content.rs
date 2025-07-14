use crate::components::util::combine_optional_class;
use leptos::prelude::*;

#[component]
pub fn BodyContent(
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=combine_optional_class("rounded-t-lg shadow-md p-6 border border-color scroll-auto overflow-y-auto", class)>
            {children()}
        </div>
    }
}
