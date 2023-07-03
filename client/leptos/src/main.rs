use leptos::*;

use plebiscite_types::*;
use plebiscite_client_webapi as api;

macro_rules! log {
    ($($args:tt)+) => {{ 
        use api::log_str;
        api::log_pfx!("Leptos", $($args)+)
    }};
}

macro_rules! log_err {
    ($($args:tt)+) => {{ 
        use api::log_str;
        api::log_pfx!("Leptos: ERROR", $($args)+)
    }};
}

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    mount_to_body(|cx| view! { cx, <MyGroups /> })
}

use api::JsonWebAPI as API;

#[component]
fn MyGroups(cx: Scope) -> impl IntoView {

    let groups = create_local_resource(cx, || (), |_| async {
        API::get_assigned_usergroups().await.map_err(std::rc::Rc::new)
    });

    view! { cx, { move || match groups.read(cx) {
        None => view! { cx, <h1>"Loading ..."</h1> }.into_view(cx),
        Some(Err(e)) => {
            log_err!("Could not get usergroups, {:?}", e);
            view! { cx, <h1>"Leptos error"</h1> }.into_view(cx)
        },
        Some(Ok(initial_groups)) => {
            let (s_groups, s_groups_set) = create_signal(cx, initial_groups);
            let txt_newgroup: NodeRef<leptos::html::Input> = create_node_ref(cx);

            let add_group_action = create_action(cx, move |g: &UsergroupData| { 
                let g: UsergroupData = g.clone();
                async move {
                    let id = API::create_usergroup(&g).await;
                    match id {
                        Ok(id) => { 
                            log!("Created group: {:?}", id);
                            s_groups_set.update(move |gs| gs.push((id, g))); 
                        },
                        Err(e) => { log_err!("Failed to create a new usergroup: {:?}", e); }
                    }
                }
            });

            let add_group = move |_| {
                let newname = txt_newgroup.get().unwrap().value().to_string();
                let g = UsergroupData { title: newname };
                add_group_action.dispatch(g);
            };

            view! { cx, 
                <ul>
                    <li>"Total: "{ move || s_groups.get().len() }</li>
                    <For each=move || { s_groups.get() }
                         view=|cx, (_, data)| view! { cx, <li>{ data.title }</li> }
                         key=|&(id, _)| id
                    />
                </ul>

                <input type="text" node_ref=txt_newgroup />
                <button on:click=add_group>"Add"</button>
            }
            .into_view(cx)
        }
    }}}

}
