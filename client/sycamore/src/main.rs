use std::ops::Deref;
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


#[component]
async fn MyGroups<G: Html>(cx: Scope<'_>) -> View<G> {

    let groups = api::get_assigned_usergroups().await;
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
                    let id = api::create_usergroup(&g).await;
                    match id {
                        Ok(id) => { 
                            log!("Created group: {:?}", id);
                            //let mut groups = <std::rc::Rc<_>>::into_inner(s_groups.take()).expect("Could not unwrap groups RC on new group creation");
                            let mut groups = s_groups.get_untracked().deref().clone();
                            groups.push((id, g)); 
                            s_groups.set(groups);
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
