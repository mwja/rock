use html5ever::{LocalName, QualName, interface::QuirksMode, local_name};
use smallvec::SmallVec;

use crate::dom::{Dom, node::NodeId};

pub enum NodeData {
    Document {
        quirks_mode: QuirksMode,
    },
    Doctype {
        name: String,
        public_id: String,
        system_id: String,
    },
    Element(Box<ElementData>),
    Text(String),
    Comment(String),
    DocumentFragment, // <template> contents, later shadow roots
    ProcessingInstruction {
        target: String,
        data: String,
    },
}

impl NodeData {
    pub fn as_element(&self) -> Option<&ElementData> {
        if let NodeData::Element(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_text(&self) -> Option<&str> {
        if let NodeData::Text(t) = self {
            Some(t)
        } else {
            None
        }
    }
}

pub struct ElementData {
    pub name: QualName,                    // namespace + local name (interned atoms)
    pub attrs: Vec<Attr>,                  // document order, small, linear scan is fine
    pub template_contents: Option<NodeId>, // only for <template>, a DocumentFragment node
    // cached for selector matching (filled when attrs change)
    pub id: Option<LocalName>,
    pub classes: SmallVec<[LocalName; 2]>,
}

pub struct Attr {
    pub name: QualName, // usually no namespace, but SVG/XLink use one
    pub value: String,
}

impl Into<Attr> for html5ever::Attribute {
    fn into(self) -> Attr {
        Attr {
            name: self.name,
            value: self.value.into(),
        }
    }
}

impl ElementData {
    pub fn attr(&self, local: &LocalName) -> Option<&str> {
        self.attrs
            .iter()
            .find(|a| a.name.local == *local)
            .map(|a| a.value.as_str())
    }

    pub fn set_attr(&mut self, name: QualName, value: String) {
        match self.attrs.iter_mut().find(|a| a.name == name) {
            Some(a) => a.value = value,
            None => self.attrs.push(Attr { name, value }),
        }
        self.refresh(); // refresh if we updated the classes etc.
    }

    /// Update the id and classes attrs to have the real data.
    pub fn add_attr_if_missing(&mut self, name: QualName, value: String) {
        if !self.attrs.iter().any(|a| a.name == name) {
            self.set_attr(name, value);
        }
    }

    fn refresh(&mut self) {
        self.id = self
            .attr(&local_name!("id"))
            .filter(|v| !v.is_empty())
            .map(LocalName::from);

        self.classes = self
            .attr(&local_name!("class"))
            .map(|v| v.split_ascii_whitespace().map(LocalName::from).collect())
            .unwrap_or_default();
    }
}

/// Extends the DOM to have useful methods for elements and adding preset
/// types of data
impl Dom {
    /// Get the element for a node, or panic. Use [try_element][Self::try_element]
    /// if you don't want panicking.
    pub fn element(&self, id: NodeId) -> &ElementData {
        match &self.data[id] {
            NodeData::Element(e) => e,
            _ => panic!("node {:?} is not an element", id),
        }
    }

    pub fn element_mut(&mut self, id: NodeId) -> &mut ElementData {
        match &mut self.data[id] {
            NodeData::Element(e) => e,
            _ => panic!("node {:?} is not an element", id),
        }
    }

    pub fn text(&self, id: NodeId) -> &str {
        match &self.data[id] {
            NodeData::Text(s) => s,
            _ => panic!("node {:?} is not an element", id),
        }
    }

    pub fn text_mut(&mut self, id: NodeId) -> &mut String {
        match &mut self.data[id] {
            NodeData::Text(s) => s,
            _ => panic!("node {:?} is not an element", id),
        }
    }

    /// Non-panicking version of [element][Self::element]
    pub fn try_element(&self, id: NodeId) -> Option<&ElementData> {
        self.data[id].as_element()
    }

    pub fn create_element(&mut self, element: ElementData) -> NodeId {
        self.create_node(NodeData::Element(Box::new(element)))
    }

    pub fn create_fragment(&mut self) -> NodeId {
        self.create_node(NodeData::DocumentFragment)
    }

    pub fn create_comment(&mut self, text: String) -> NodeId {
        self.create_node(NodeData::Comment(text))
    }

    pub fn create_pi(&mut self, target: String, data: String) -> NodeId {
        self.create_node(NodeData::ProcessingInstruction { target, data })
    }

    pub fn create_text(&mut self, text: String) -> NodeId {
        self.create_node(NodeData::Text(text))
    }

    pub fn create_doctype(&mut self, name: String, public_id: String, system_id: String) -> NodeId {
        self.create_node(NodeData::Doctype {
            name,
            public_id,
            system_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use html5ever::ns;

    fn qual(local: &str) -> QualName {
        QualName::new(None, ns!(), LocalName::from(local))
    }

    fn div() -> ElementData {
        ElementData {
            name: qual("div"),
            attrs: Vec::new(),
            template_contents: None,
            id: None,
            classes: Default::default(),
        }
    }

    #[test]
    fn set_attr_refreshes_id_and_classes() {
        let mut el = div();
        el.set_attr(qual("id"), "main".into());
        el.set_attr(qual("class"), "  a   b ".into());
        assert_eq!(el.id.as_deref(), Some("main"));
        assert_eq!(
            el.classes.as_slice(),
            [LocalName::from("a"), LocalName::from("b")]
        );
    }

    #[test]
    fn add_attr_if_missing_does_not_overwrite() {
        let mut el = div();
        el.set_attr(qual("id"), "first".into());
        el.add_attr_if_missing(qual("id"), "second".into());
        el.add_attr_if_missing(qual("title"), "t".into());
        assert_eq!(el.attr(&local_name!("id")), Some("first"));
        assert_eq!(el.attr(&local_name!("title")), Some("t"));
    }

    #[test]
    fn try_element_is_none_for_text() {
        let mut dom = Dom::new();
        let text = dom.create_text("hi".into());
        let el = dom.create_element(div());
        assert!(dom.try_element(text).is_none());
        assert!(dom.try_element(el).is_some());
    }
}
