use crate::Comment;
use std::{borrow::Cow, cell::RefCell, rc::Rc};

use leptos_reactive::{create_effect, Scope};

use crate::{mount_child, Component, IntoNode, MountKind, Node};

/// The internal representation of the [`DynChild`] core-component.
#[derive(Debug)]
pub struct DynChildRepr {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    document_fragment: web_sys::DocumentFragment,
    opening: Comment,
    child: Rc<RefCell<Box<Node>>>,
    closing: Comment,
}

impl Default for DynChildRepr {
    fn default() -> Self {
        let (opening, closing) = {
            let (opening, closing) = (
                Comment::new(Cow::Borrowed("<DynChild>")),
                Comment::new(Cow::Borrowed("</DynChild>")),
            );

            (opening, closing)
        };

        #[cfg(all(target_arch = "wasm32", feature = "web"))]
        let document_fragment = {
            let fragment = gloo::utils::document().create_document_fragment();

            // Insert the comments into the document fragment
            // so they can serve as our references when inserting
            // future nodes
            fragment
                .append_with_node_2(&opening.node, &closing.node)
                .expect("append to not err");

            fragment
        };

        Self {
            #[cfg(all(target_arch = "wasm32", feature = "web"))]
            document_fragment,
            opening,
            child: Default::default(),
            closing,
        }
    }
}

impl DynChildRepr {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    pub(crate) fn get_web_sys_node(&self) -> web_sys::Node {
        use wasm_bindgen::JsCast;

        self.document_fragment.clone().unchecked_into()
    }
}

/// Represents any [`Node`] that can change over time.
pub struct DynChild<CF, N>
where
    CF: Fn() -> N + 'static,
    N: IntoNode,
{
    child_fn: CF,
}

impl<CF, N> DynChild<CF, N>
where
    CF: Fn() -> N + 'static,
    N: IntoNode,
{
    /// Creates a new dynamic child which will re-render whenever it's
    /// signal dependencies change.
    pub fn new(child_fn: CF) -> Self {
        Self { child_fn }
    }
}

impl<CF, N> IntoNode for DynChild<CF, N>
where
    CF: Fn() -> N + 'static,
    N: IntoNode,
{
    #[instrument(level = "trace", skip_all)]
    fn into_node(self, cx: Scope) -> crate::Node {
        let Self { child_fn } = self;

        let component = DynChildRepr::default();

        #[cfg(all(target_arch = "wasm32", feature = "web"))]
        let closing = component.closing.node.0.clone();
        let child = component.child.clone();

        let span = tracing::Span::current();

        create_effect(cx, move |_| {
            let _guard = span.enter();
            let _guard = trace_span!("DynChild reactive").entered();

            let new_child = child_fn().into_node(cx);

            mount_child(MountKind::Component(&closing), &new_child);

            **child.borrow_mut() = new_child;
        });

        Node::CoreComponent(crate::CoreComponent::DynChild(component))
    }
}