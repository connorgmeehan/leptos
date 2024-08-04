use leptos::tachys::view::{
    any_view::AnyView,
    fragment::{Fragment, IntoFragment},
    Render,
};

use super::{
    renderer::BevyRenderer,
    view::{BevyIntoAny, BevyView, IntoBevyView},
};

use std::{
    fmt::{self, Debug},
    sync::Arc,
};

/// The most common type for the `children` property on components,
/// which can only be called once.
///
/// This does not support iterating over individual nodes within the children.
/// To iterate over children, use [`ChildrenFragment`].
pub type BevyChildren = Box<dyn FnOnce() -> AnyView<BevyRenderer> + Send>;

/// A type for the `children` property on components that can be called only once,
/// and provides a collection of all the children passed to this component.
pub type BevyChildrenFragment =
    Box<dyn FnOnce() -> Fragment<BevyRenderer> + Send>;

/// A type for the `children` property on components that can be called
/// more than once.
pub type BevyChildrenFn = Arc<dyn Fn() -> AnyView<BevyRenderer> + Send + Sync>;

/// A type for the `children` property on components that can be called more than once,
/// and provides a collection of all the children passed to this component.
pub type BevyChildrenFragmentFn =
    Arc<dyn Fn() -> Fragment<BevyRenderer> + Send>;

/// A type for the `children` property on components that can be called
/// more than once, but may mutate the children.
pub type BevyChildrenFnMut = Box<dyn FnMut() -> AnyView<BevyRenderer> + Send>;

/// A type for the `children` property on components that can be called more than once,
/// but may mutate the children, and provides a collection of all the children
/// passed to this component.
pub type BevyChildrenFragmentMut =
    Box<dyn FnMut() -> Fragment<BevyRenderer> + Send>;

// This is to still support components that accept `Box<dyn Fn() -> AnyView>` as a children.
type BevyBoxedChildrenFn = Box<dyn Fn() -> AnyView<BevyRenderer> + Send>;

/// This trait can be used when constructing a component that takes children without needing
/// to know exactly what children type the component expects. This is used internally by the
/// `view!` macro implementation, and can also be used explicitly when using the builder syntax.
///
///
/// Different component types take different types for their `children` prop, some of which cannot
/// be directly constructed. Using `ToChildren` allows the component user to pass children without
/// explicity constructing the correct type.
///
/// ## Examples
///
/// ```
/// # use leptos::prelude::*;
/// # use leptos::html::p;
/// # use leptos::IntoBevyView;
/// # use leptos_macro::component;
/// # use leptos::children::ToChildren;
/// use leptos::context::{Provider, ProviderProps};
/// use leptos::control_flow::{Show, ShowProps};
///
/// #[component]
/// fn App() -> impl IntoBevyView {
///     (
///       Provider(
///         ProviderProps::builder()
///             .children(ToChildren::to_children(|| {
///                 p().child("Foo")
///             }))
///             // ...
///            .value("Foo")
///            .build(),
///        ),
///        Show(
///          ShowProps::builder()
///             .children(ToChildren::to_children(|| {
///                 p().child("Foo")
///             }))
///             // ...
///             .when(|| true)
///             .fallback(|| p().child("foo"))
///             .build(),
///        )
///     )
/// }
pub trait ToChildren<F> {
    /// Convert the provided type to (generally a closure) to Self (generally a "children" type,
    /// e.g., [Children]). See the implementations to see exactly which input types are supported
    /// and which "children" type they are converted to.
    fn to_children(f: F) -> Self;
}

impl<F, C> ToChildren<F> for BevyChildren
where
    F: FnOnce() -> C + Send + 'static,
    C: Render<BevyRenderer> + BevyIntoAny<BevyRenderer> + Send + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for BevyChildrenFn
where
    F: Fn() -> C + Send + Sync + 'static,
    C: Render<BevyRenderer> + Send + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Arc::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for BevyChildrenFnMut
where
    F: Fn() -> C + Send + 'static,
    C: Render<BevyRenderer> + Send + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for BevyBoxedChildrenFn
where
    F: Fn() -> C + Send + 'static,
    C: Render<BevyRenderer> + Send + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for BevyChildrenFragment
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoFragment<BevyRenderer>,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_fragment())
    }
}

impl<F, C> ToChildren<F> for BevyChildrenFragmentFn
where
    F: Fn() -> C + Send + 'static,
    C: IntoFragment<BevyRenderer>,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Arc::new(move || f().into_fragment())
    }
}

impl<F, C> ToChildren<F> for BevyChildrenFragmentMut
where
    F: FnMut() -> C + Send + 'static,
    C: IntoFragment<BevyRenderer>,
{
    #[inline]
    fn to_children(mut f: F) -> Self {
        Box::new(move || f().into_fragment())
    }
}

/// New-type wrapper for a function that returns a view with `From` and `Default` traits implemented
/// to enable optional props in for example `<Show>` and `<Suspense>`.
#[derive(Clone)]
pub struct ViewFn(
    Arc<dyn Fn() -> AnyView<BevyRenderer> + Send + Sync + 'static>,
);

impl Default for ViewFn {
    fn default() -> Self {
        Self(Arc::new(|| ().into_any()))
    }
}

impl<F, C> From<F> for ViewFn
where
    F: Fn() -> C + Send + Sync + 'static,
    C: Render<BevyRenderer> + Send + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(move || value().into_any()))
    }
}

impl ViewFn {
    /// Execute the wrapped function
    pub fn run(&self) -> AnyView<BevyRenderer> {
        (self.0)()
    }
}

/// New-type wrapper for a function, which will only be called once and returns a view with `From` and
/// `Default` traits implemented to enable optional props in for example `<Show>` and `<Suspense>`.
pub struct ViewFnOnce(
    Box<dyn FnOnce() -> AnyView<BevyRenderer> + Send + 'static>,
);

impl Default for ViewFnOnce {
    fn default() -> Self {
        Self(Box::new(|| ().into_any()))
    }
}

impl<F, C> From<F> for ViewFnOnce
where
    F: FnOnce() -> C + Send + 'static,
    C: Render<BevyRenderer> + Send + 'static,
{
    fn from(value: F) -> Self {
        Self(Box::new(move || value().into_any()))
    }
}

impl ViewFnOnce {
    /// Execute the wrapped function
    pub fn run(self) -> AnyView<BevyRenderer> {
        (self.0)()
    }
}

/// A typed equivalent to [`Children`], which takes a generic but preserves type information to
/// allow the compiler to optimize the view more effectively.
pub struct BevyTypedChildren<T>(Box<dyn FnOnce() -> BevyView<T> + Send>);

impl<T> BevyTypedChildren<T> {
    pub fn into_inner(self) -> impl FnOnce() -> BevyView<T> + Send {
        self.0
    }
}

impl<F, C> ToChildren<F> for BevyTypedChildren<C>
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoBevyView,
{
    #[inline]
    fn to_children(f: F) -> Self {
        BevyTypedChildren(Box::new(move || f().into_view()))
    }
}

/// A typed equivalent to [`ChildrenMut`], which takes a generic but preserves type information to
/// allow the compiler to optimize the view more effectively.
pub struct TypedChildrenMut<T>(Box<dyn FnMut() -> BevyView<T> + Send>);

impl<T> Debug for TypedChildrenMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TypedChildrenMut").finish()
    }
}

impl<T> TypedChildrenMut<T> {
    pub fn into_inner(self) -> impl FnMut() -> BevyView<T> + Send {
        self.0
    }
}

impl<F, C> ToChildren<F> for TypedChildrenMut<C>
where
    F: FnMut() -> C + Send + 'static,
    C: IntoBevyView,
{
    #[inline]
    fn to_children(mut f: F) -> Self {
        TypedChildrenMut(Box::new(move || f().into_view()))
    }
}

/// A typed equivalent to [`ChildrenFn`], which takes a generic but preserves type information to
/// allow the compiler to optimize the view more effectively.
pub struct TypedChildrenFn<T>(Arc<dyn Fn() -> BevyView<T> + Send + Sync>);

impl<T> Debug for TypedChildrenFn<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TypedChildrenFn").finish()
    }
}

impl<T> TypedChildrenFn<T> {
    pub fn into_inner(self) -> Arc<dyn Fn() -> BevyView<T> + Send + Sync> {
        self.0
    }
}

impl<F, C> ToChildren<F> for TypedChildrenFn<C>
where
    F: Fn() -> C + Send + Sync + 'static,
    C: IntoBevyView,
{
    #[inline]
    fn to_children(f: F) -> Self {
        TypedChildrenFn(Arc::new(move || f().into_view()))
    }
}
