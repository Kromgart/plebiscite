use sycamore::prelude::*;
//use lazy_static::lazy_static;


#[derive(Clone, PartialEq, Eq)]
struct Group {
    name: String,
    id: i64,
}

fn main() {

    sycamore::render(|cx| view! { cx,
        MyGroups { }
    });
}

/*lazy_static! {
    static ref TEST_GROUPS: Vec<Group> = vec! [
        Group { name: String::from("cycling"), id: 1 },
        Group { name: String::from("airsoftaa"), id: 2 },
    ];
}*/

#[component]
fn MyGroups<G:Html>(cx: Scope) -> View<G> {

    let groups = create_signal(cx, vec![
        Group { name: String::from("cycling"), id: 1 },
        Group { name: String::from("airsoft"), id: 2 },
    ]);

    view! { cx, ul { 
        Keyed (
            iterable=groups,
            view=|cx, g| view! { cx, li { (g.name) } },
            key=|g| g.id
        ) 
    }}

}
