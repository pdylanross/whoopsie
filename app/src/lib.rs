#[macro_use]
extern crate macros;

mod components;
pub mod config;
mod pages;
#[cfg(feature = "ssr")]
pub mod state;
pub mod tracing;
pub mod types;

use crate::pages::home::*;
use crate::pages::login::LoginPage;
use crate::pages::not_found::NotFoundPage;
use crate::types::DbFactory;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use std::sync::Arc;

pub type DbFactoryPointer = Arc<dyn DbFactory + Send + Sync>;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <AutoReload options=options.clone()/>
                    <HydrationScripts options/>
                    <MetaTags/>
                </head>
                <body>
                    <App/>
                </body>
            </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/whoops.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| NotFoundPage>
                    <HomeRoutes/>
                    <Route path=path!("") view=LoginPage/>
                </Routes>
            </main>
        </Router>
    }
}
