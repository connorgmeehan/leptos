use leptos::{context::provide_context, reactive_graph::owner::Owner, tachys::reactive_graph::OwnedView};

use super::{children::BevyTypedChildren, view::IntoBevyView};


#[allow(non_snake_case)]
pub fn Provider<T, TChildren>(
    value: T,
    children: BevyTypedChildren<TChildren>,
) -> impl IntoBevyView
where
    T: Send + Sync + 'static,
    TChildren: IntoBevyView,
{
    let owner = Owner::current()
        .expect("no current reactive Owner found")
        .child();
    let children = children.into_inner();
    let children = owner.with(|| {
        provide_context(value);
        children()
    });
    OwnedView::new_with_owner(children, owner)
}
