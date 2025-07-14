use crate::components::util::combine_optional_class;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar(#[prop(optional)] class: Option<&'static str>) -> impl IntoView {
    let navbar_hr_class =
        "rounded-lg border-1 border-accent-400 w-3/4 m-2 outline-1 outline-accent-600 shadow-lg";

    view! {
        <div class=combine_optional_class("rounded-r-lg shadow-md p-6 border border-color", class)>
            <div class="flex flex-col justify-center items-center w-full">
                <NavbarItem text="Home" href="/home"/>
                <NavbarItem text="About" href="/home/about"/>
                <NavbarItem text="Scroll" href="/home/scroll"/>
                <hr class={navbar_hr_class}/>
                <NavbarItem text="Logout" href="/"/>
            </div>
        </div>
    }
}

#[component]
fn NavbarItem(#[prop(into)] text: String, #[prop(into)] href: String) -> impl IntoView {
    view! {
        <div class="text-center btn-outline m-1 w-full">
            <A href={href}><p>{text}</p></A>
        </div>
    }
}
