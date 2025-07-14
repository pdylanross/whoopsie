use crate::components::monitor::MonitorList;
use crate::components::structural::body_content::BodyContent;
use crate::components::structural::header::PageHeader;
use crate::components::structural::navbar::Navbar;
use crate::components::util::lorem_ipsum;
use crate::types::{get_monitors, Monitor};
use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Route};
use leptos_router::path;

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-dvh">
            <PageHeader class="col-span-2"/>

            <div class="flex flex-row size-full">
                <Navbar class="w-32 h-full"/>
                <BodyContent class="size-full">
                    <Outlet/>
                </BodyContent>
            </div>
        </div>

    }
}

#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <div>
            <h1>"About"</h1>
        </div>
    }
}

#[component]
fn HomeLandingPage() -> impl IntoView {
    let monitors: LocalResource<Result<Vec<Monitor>, ServerFnError>> =
        LocalResource::new(async move || {
            let m = get_monitors().await?;
            Ok(m)
        });

    view! {
        <div>
            <h1>"Home"</h1>
            <Suspense fallback=move || view! {<div>Loading...</div>}>
                {move || {
                    monitors.get().map(|monitor| {
                        match monitor {
                            Ok(monitors) => {
                                view! {
                                    <MonitorList monitors=monitors/>
                                }
                            }.into_any(),
                            Err(err) => view! {
                                <div class="text-red-500">{err.to_string()}</div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn Scroll() -> impl IntoView {
    let (data, _set_data) = signal(0..1000);

    view! {
        <For each=move || data.get()
            key=|state| *state
            let(_child)
        >
            <div class="w-full margin-2 bg-primary-200">
                {lorem_ipsum()}
            </div>
        </For>
    }
}

#[component(transparent)]
pub fn HomeRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    view! {
      <ParentRoute path=path!("home") view=HomePage>
        <Route path=path!("") view=HomeLandingPage/>
        <Route path=path!("about") view=AboutPage/>
        <Route path=path!("scroll") view=Scroll/>
      </ParentRoute>
    }
    .into_inner()
}
