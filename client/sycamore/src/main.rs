use sycamore::prelude::*;


fn main() {

    sycamore::render(|cx| view! { cx,
        MyGroups { }
    });
}


#[component]
async fn MyGroups<G: Html>(cx: Scope<'_>) -> View<G> {

    let groups = plebiscite_client_webapi::get_assigned_usergroups().await;
    match groups {
        Ok(groups) => {

            let groups = create_signal(cx, groups);

            view! { cx, ul { 
                Keyed (
                    iterable=groups,
                    view=|cx, g| view! { cx, li { (g.title) } },
                    key=|g| g.usergroup_id
                ) 
            }}
        },
        Err(e) => {
            println!("{:?}", e);

            view! { cx, h1 { "error" } }
        }
    }

}
