use crate::components::util::combine_optional_class;
use leptos::prelude::*;

#[component]
pub fn PageHeader(#[prop(optional)] class: Option<&'static str>) -> impl IntoView {
    view! {
        <div class=combine_optional_class("rounded-b-lg shadow-md p-6 border border-color flex", class)>
        <img src="/logo.png" class="h-16 w-16"/>

        <h1>"whoopsie"</h1>
        </div>
    }
}
