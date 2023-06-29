use sycamore::prelude::*;

use plebiscite_types::*;
use plebiscite_client_webapi as api;


macro_rules! log {
    ($($args:tt)+) => {{ 
        use api::log_str;
        api::log_pfx!("Sycamore", $($args)+)
    }};
}

macro_rules! log_err {
    ($($args:tt)+) => {{ 
        use api::log_str;
        api::log_pfx!("Sycamore: ERROR", $($args)+)
    }};
}

fn main() {

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    sycamore::render(|cx| view! { cx,
        MyGroups { }
    });
}

use api::JsonWebAPI as API;

#[component]
async fn MyGroups<G: Html>(cx: Scope<'_>) -> View<G> {

    let groups = API::get_assigned_usergroups().await;
    match groups {
        Ok(groups) => {

            let txt_new_group = create_signal(cx, String::new());
            let s_groups = create_signal(cx, groups);

            view! { cx, 
                ul { 
                    Keyed (
                        iterable=s_groups,
                        view=|cx, (_, data)| view! { cx, li { (data.title) } },
                        key=|&(id, _)| id
                    ) 
                }

                input(type="text", bind:value=txt_new_group)

                button(on:click=|_| async { 
                    let g = UsergroupData { title: txt_new_group.to_string() };
                    let id = API::create_usergroup(&g).await;
                    match id {
                        Ok(id) => { 
                            log!("Created group: {:?}", id);
                            s_groups.modify().push((id, g)); 
                        },
                        Err(e) => { log_err!("Failed to create a new usergroup: {:?}", e); }
                    }
                }) { "Add" }
            }
        },
        Err(e) => {
            log_err!("Could not get usergroups, {:?}", e);

            view! { cx, h1 { "Sycamore error" } }
        }
    }

}
