use gpui::*;
use std::ops::Range;

#[allow(unused_imports)]
pub use gpui::{ScrollStrategy, UniformList, UniformListScrollHandle, uniform_list};

#[allow(dead_code)]
pub fn virtual_list<R, V>(
    id: impl Into<ElementId>,
    count: usize,
    cx: &Context<V>,
    render: impl Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R> + 'static,
) -> UniformList
where
    R: IntoElement,
    V: Render,
{
    uniform_list(id, count, cx.processor(render))
}
