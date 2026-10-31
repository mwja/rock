use std::cell::{Ref, RefCell};

use html5ever::{
    LocalName, Namespace, QualName,
    interface::{ElemName, NodeOrText, TreeSink},
};
use tracing::error;

use crate::dom::{Dom, ElementData, NodeId};

/// Lightweight wrapper around the qualified names from html5ever.
#[derive(Debug)]
pub struct NameRef<'a>(Ref<'a, QualName>);

impl ElemName for NameRef<'_> {
    fn ns(&self) -> &Namespace {
        &self.0.ns
    }
    fn local_name(&self) -> &LocalName {
        &self.0.local
    }
}

pub struct DomSink {
    dom: RefCell<Dom>,
}

impl DomSink {
    pub fn new(dom: Dom) -> Self {
        Self {
            dom: RefCell::new(dom),
        }
    }

    fn append_node_or_text(&self, parent: NodeId, node_or_text: NodeOrText<NodeId>) {
        match node_or_text {
            NodeOrText::AppendText(text) => {
                self.dom.borrow_mut().append_text(parent, text.into());
            }
            NodeOrText::AppendNode(node) => self.dom.borrow_mut().append_child(parent, node),
        }
    }

    fn append_node_or_text_before(&self, sibling: NodeId, node_or_text: NodeOrText<NodeId>) {
        match node_or_text {
            NodeOrText::AppendText(text) => {
                self.dom
                    .borrow_mut()
                    .append_text_before(sibling, text.into());
            }
            NodeOrText::AppendNode(node) => self.dom.borrow_mut().append_before(sibling, node),
        }
    }
}

impl TreeSink for DomSink {
    type Handle = NodeId;

    type Output = Dom;

    type ElemName<'a>
        = NameRef<'a>
    where
        Self: 'a;

    fn finish(self) -> Self::Output {
        self.dom.into_inner()
    }

    fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
        error!(target: "parser", "failed to parse with error: {msg}");
    }

    fn get_document(&self) -> Self::Handle {
        self.dom.borrow().root()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        NameRef(Ref::map(self.dom.borrow(), |dom| {
            &dom.element(*target).name
        }))
    }

    fn create_element(
        &self,
        name: html5ever::QualName,
        attrs: Vec<html5ever::Attribute>,
        flags: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        let mut dom = self.dom.borrow_mut();

        let id = dom.create_element(ElementData {
            name,
            attrs: attrs.into_iter().map(Into::into).collect(),
            template_contents: None,
            id: None,
            classes: Default::default(),
        });

        if flags.template {
            let frag = dom.create_fragment();
            dom.element_mut(id).template_contents = Some(frag);
        }

        id
    }

    fn create_comment(&self, text: html5ever::tendril::StrTendril) -> Self::Handle {
        self.dom.borrow_mut().create_comment(text.into())
    }

    fn create_pi(
        &self,
        target: html5ever::tendril::StrTendril,
        data: html5ever::tendril::StrTendril,
    ) -> Self::Handle {
        self.dom.borrow_mut().create_pi(target.into(), data.into())
    }

    fn append(&self, parent: &Self::Handle, child: html5ever::interface::NodeOrText<Self::Handle>) {
        self.append_node_or_text(*parent, child);
    }

    // This is (based on my brief searching) when the spec requires an element
    // be forcibly moved outside of its tree. That is, for example a <table>
    // with stray text in it (not allowed)
    //
    // In that case, element is the table, prev_element is the element above the
    // table (for fragments). So if the table has a parent, we append it before
    // the table. If it doesn't, we append it above the table.
    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        // Does the given element already have a parent? if so insert it after the
        // element. Otherwise just append it.
        let has_parent = self.dom.borrow().parent(*element).is_some();
        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        name: html5ever::tendril::StrTendril,
        public_id: html5ever::tendril::StrTendril,
        system_id: html5ever::tendril::StrTendril,
    ) {
        let root = self.dom.borrow().root();
        let mut dom = self.dom.borrow_mut();
        let doctype = dom.create_doctype(name.into(), public_id.into(), system_id.into());
        dom.append_child(root, doctype);
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        self.dom
            .borrow()
            .element(*target)
            .template_contents
            .expect("get_template_contents called on a non-template")
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, mode: html5ever::interface::QuirksMode) {
        self.dom.borrow_mut().set_quirks_mode(mode);
    }

    fn append_before_sibling(
        &self,
        sibling: &Self::Handle,
        new_node: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        self.append_node_or_text_before(*sibling, new_node);
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let mut dom = self.dom.borrow_mut();
        let el = dom.element_mut(*target);
        for a in attrs {
            el.add_attr_if_missing(a.name, a.value.to_string());
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.dom.borrow_mut().detach(*target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut dom = self.dom.borrow_mut();
        let children: Vec<_> = dom.children(*node).collect();
        for child in children {
            dom.append_child(*new_parent, child);
        }
    }
}
