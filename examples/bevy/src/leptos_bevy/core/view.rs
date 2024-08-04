use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use leptos::tachys::{
    renderer::Renderer,
    view::{
        any_view::{insert_before_this, mount_any, unmount_any, AnyView, AnyViewState, IntoAny},
        Mountable, Render,
    },
};

use super::renderer::BevyRenderer;

pub struct BevyView<T>(T)
where
    T: Sized;

impl<T> BevyView<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

pub trait IntoBevyView
where
    Self: Sized + Render<BevyRenderer> + Send,
{
    fn into_view(self) -> BevyView<Self>;
}

impl<T> IntoBevyView for T
where
    T: Sized + Render<BevyRenderer> + Send, //+ AddAnyAttr<Dom>,
{
    fn into_view(self) -> BevyView<Self> {
        BevyView(self)
    }
}

impl<T: IntoBevyView> Render<BevyRenderer> for BevyView<T> {
    type State = T::State;

    fn build(self) -> Self::State {
        self.0.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.0.rebuild(state)
    }
}

/// Allows converting some view into [`AnyView`].
pub trait BevyIntoAny<R>
where
    R: Renderer,
{
    /// Converts the view into a type-erased [`AnyView`].
    fn into_any(self) -> AnyView<R>;
}

impl<T, R> BevyIntoAny<R> for T
where
    T: Send,
    T: Render<R> + 'static,
    T::State: 'static,
    R: Renderer + 'static,
{
    // inlining allows the compiler to remove the unused functions
    // i.e., doesn't ship HTML-generating code that isn't used
    #[inline(always)]
    fn into_any(self) -> AnyView<R> {
        let value = Box::new(self) as Box<dyn Any + Send>;
        let build = |value: Box<dyn Any>| {
            let value = value
                .downcast::<T>()
                .expect("AnyView::build couldn't downcast");
            let state = Box::new(value.build());

            AnyViewState {
                type_id: TypeId::of::<T>(),
                state,
                rndr: PhantomData,
                mount: mount_any::<R, T>,
                unmount: unmount_any::<R, T>,
                insert_before_this: insert_before_this::<R, T>,
            }
        };

        let rebuild = |new_type_id: TypeId,
                       value: Box<dyn Any>,
                       state: &mut AnyViewState<R>| {
            let value = value
                .downcast::<T>()
                .expect("AnyView::rebuild couldn't downcast value");
            if new_type_id == state.type_id {
                let state = state
                    .state
                    .downcast_mut()
                    .expect("AnyView::rebuild couldn't downcast state");
                value.rebuild(state);
            } else {
                let mut new = value.into_any().build();
                state.insert_before_this(&mut new);
                state.unmount();
                *state = new;
            }
        };
        AnyView {
            type_id: TypeId::of::<T>(),
            value,
            build,
            rebuild,
        }
    }
}
