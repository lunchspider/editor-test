use std::time::Duration;

use leptos::{html::Div, *};
use leptos_meta::*;
use leptos_router::*;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::{Element, MutationObserver, MutationObserverInit, MutationRecord};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,
        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Debug, Clone)]
struct EditorElement {
    pub id: Uuid,
    pub content: String,
}

#[component]
fn Render(cx: Scope, element: EditorElement) -> impl IntoView {
    view! { cx,
        <div id = element.id.to_string()>
            {element.content}
        </div>
    }
}

fn handle_mutation(
    mutation_records: Vec<MutationRecord>,
    data: ReadSignal<Vec<EditorElement>>,
    set_data: WriteSignal<Vec<EditorElement>>,
) {
    for mutation_record in mutation_records {
        let target = mutation_record.target().unwrap();
        match mutation_record.type_().as_str() {
            "characterData" => {
                let el = target.parent_element().unwrap();
                if let Ok(id) = el.id().parse::<Uuid>() {
                    let index = data()
                        .into_iter()
                        .position(|element| element.id == id)
                        .unwrap_or(0);
                    log!("{}", index);
                    set_data
                        .update(|data| data[index].content = el.text_content().unwrap_or_default());
                }
            }
            "childList" => {
                let el: Element = target.unchecked_into();
                if el.id().as_str() == "editor" {
                    continue;
                }
                if let Ok(_) = el.id().parse::<Uuid>() {
                    continue;
                }
                log!("{}", el.id());
                let editor_element = EditorElement {
                    id: Uuid::new_v4(),
                    content: el.text_content().unwrap_or_default(),
                };
                el.set_id(&editor_element.id.to_string());
                set_data.update(|data| data.push(editor_element));
            }
            anything_else => {
                // ignoring anything else for now!
                log!("{}", anything_else)
            }
        }
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let editor_ref = create_node_ref::<Div>(cx);
    let (data, set_data) = create_signal(
        cx,
        vec![EditorElement {
            id: Uuid::new_v4(),
            content: "first element".to_string(),
        }],
    );

    editor_ref.on_load(cx, move |editor_ref| {
        let mutation_callback: Closure<dyn FnMut(_, _)> = Closure::new(
            move |mutation_records: Vec<MutationRecord>, _observer: MutationObserver| {
                handle_mutation(mutation_records, data, set_data);
            },
        );
        let mutation_observer =
            MutationObserver::new(mutation_callback.into_js_value().dyn_ref().unwrap())
                .expect("Cannot create new mutation observer");
        mutation_observer
            .observe_with_options(
                &editor_ref,
                MutationObserverInit::new()
                    // child attributes or editor attributes chanding
                    .attributes(true)
                    // a new child get created or deleted
                    .child_list(true)
                    // user typed something
                    .character_data(true)
                    .character_data_old_value(true)
                    .subtree(true),
            )
            .expect("Cannot connect mutation observer");
    });

    // using this as a test to creates nodes directly from codes
    // and how it interacts with a browser controlled component with contentEditable
    #[cfg(not(feature = "ssr"))]
    set_timeout(
        move || {
            set_data.update(|value| {
                value.push(EditorElement {
                    id: Uuid::new_v4(),
                    content: "this is a test!".to_string(),
                })
            });
        },
        Duration::from_secs(2),
    );

    // Creates a reactive value to update the button

    view! { cx,
        <div _ref = editor_ref contentEditable = "true" id = "editor">
            <For
                each =  data
                key = |el| { el.id }
                view = move |cx, element : EditorElement| {view! { cx , <Render element />}}/>
        </div>
    }
}
