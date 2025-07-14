use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="grid place-items-center size-full bg-secondary-500 h-screen">
            <A href="/home">
                <div class="btn-primary w-32 h-32">Login</div>
            </A>
        </div>
    }
}
