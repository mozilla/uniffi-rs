/// UDL items -- imagine the component interface code outputing a Vec of these things

use std::convert::*;

// Seems like there should be a nicer way to spell out this nested enum, any suggestions?

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum UdlItem {
    Type(TypeItem),
    // We would also add:
    // Function(FunctionItem),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum TypeItem {
    I32,
    String,
    Option(Box<TypeItem>),
    Record(RecordDef),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct RecordDef {
    pub name: String,
    pub fields: Vec<(String, TypeItem)>,
}

impl Into<UdlItem> for TypeItem {
    fn into(self) -> UdlItem {
        UdlItem::Type(self)
    }
}

impl UdlItem {
    pub fn get_children(&self) -> Vec<UdlItem> {
        match self {
            Self::Type(TypeItem::Option(inner)) => vec![inner.as_ref().clone().into()],
            Self::Type(TypeItem::Record(RecordDef { fields, .. })) => {
                fields.iter().map(|f| f.1.clone().into()).collect()
            }
            _ => Vec::new(),
        }
    }

    pub fn into_self_and_descendents(self) -> Vec<UdlItem> {
        let mut result = vec![self.clone()];
        self.collect_descendents(&mut result);
        result
    }
    
    fn collect_descendents(&self, result: &mut Vec<UdlItem>) {
        let mut children = self.get_children();
        children.iter().for_each(|t| t.collect_descendents(result));
        result.append(&mut children);
    }
}
