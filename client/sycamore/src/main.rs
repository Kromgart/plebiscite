use sycamore::prelude::*;

use plebiscite_types::*;
use plebiscite_client_webapi as api;

fn main() {

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
                            //groups.push((id, g)); 
                        },
                        Err(e) => { println!("Sycamore error: {:?}", e); }
                    }
                }) { "Add" }
            }
        },
        Err(e) => {
            println!("Sycamore error: {:?}", e);

            view! { cx, h1 { "Sycamore error" } }
        }
    }

}
