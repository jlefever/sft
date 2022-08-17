use std::hash::Hash;
use std::path::Path;

use bimap::BiHashMap;
use itertools::Itertools;

use crate::collections::KindedEdgeBag;
use crate::io::{Entry, EntryReader, Ticket};

#[derive(Debug)]
pub enum ParseErr {
    UnknownAnchorKind(String),
    UnknownEdgeKind(String),
    UnknownFactName(String),
    UnknownFunctionKind(String),
    UnknownRecordKind(String),
    UnknownSumKind(String),
    UnknownVariableKind(String),
    UnknownComplete(String),
    UnknownNodeKind(String),
    MissingFact(&'static str),
    ExpectedInt,
    SequencingErr(NodeIndex, Box<ParseErr>),
}

type Result<T> = std::result::Result<T, ParseErr>;

#[derive(Default, Debug, Eq, Hash, PartialEq)]
pub enum EdgeKind {
    Aliases,
    AliasesRoot,
    Childof,
    ChildofContext,
    Completedby,
    Completes,
    CompletesUniquely,
    Defines,
    DefinesBinding,
    Documents,
    ExtendsPrivate,
    ExtendsProtected,
    ExtendsPublic,
    ExtendsPublicVirtual,
    Instantiates,
    InstantiatesSpeculative,
    Overrides,
    OverridesRoot,
    Param(u8),
    #[default]
    Ref,
    RefCall,
    RefCallImplicit,
    RefDoc,
    RefExpands,
    RefExpandsTransitive,
    RefId,
    RefImplicit,
    RefIncludes,
    RefInit,
    RefInitImplicit,
    RefQueries,
    RefWrites,
    RefWritesImplicit,
    Specializes,
    SpecializesSpeculative,
    Typed,
    Undefines,
}

impl TryFrom<&str> for EdgeKind {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "/kythe/edge/aliases" => EdgeKind::Aliases,
            "/kythe/edge/aliases/root" => EdgeKind::AliasesRoot,
            "/kythe/edge/childof" => EdgeKind::Childof,
            "/kythe/edge/childof/context" => EdgeKind::ChildofContext,
            "/kythe/edge/completedby" => EdgeKind::Completedby,
            "/kythe/edge/completes" => EdgeKind::Completes,
            "/kythe/edge/completes/uniquely" => EdgeKind::CompletesUniquely,
            "/kythe/edge/defines" => EdgeKind::Defines,
            "/kythe/edge/defines/binding" => EdgeKind::DefinesBinding,
            "/kythe/edge/documents" => EdgeKind::Documents,
            "/kythe/edge/extends/private" => EdgeKind::ExtendsPrivate,
            "/kythe/edge/extends/protected" => EdgeKind::ExtendsProtected,
            "/kythe/edge/extends/public" => EdgeKind::ExtendsPublic,
            "/kythe/edge/extends/public/virtual" => EdgeKind::ExtendsPublicVirtual,
            "/kythe/edge/instantiates" => EdgeKind::Instantiates,
            "/kythe/edge/instantiates/speculative" => EdgeKind::InstantiatesSpeculative,
            "/kythe/edge/overrides" => EdgeKind::Overrides,
            "/kythe/edge/overrides/root" => EdgeKind::OverridesRoot,
            "/kythe/edge/ref" => EdgeKind::Ref,
            "/kythe/edge/ref/call" => EdgeKind::RefCall,
            "/kythe/edge/ref/call/implicit" => EdgeKind::RefCallImplicit,
            "/kythe/edge/ref/doc" => EdgeKind::RefDoc,
            "/kythe/edge/ref/expands" => EdgeKind::RefExpands,
            "/kythe/edge/ref/expands/transitive" => EdgeKind::RefExpandsTransitive,
            "/kythe/edge/ref/id" => EdgeKind::RefId,
            "/kythe/edge/ref/implicit" => EdgeKind::RefImplicit,
            "/kythe/edge/ref/includes" => EdgeKind::RefIncludes,
            "/kythe/edge/ref/init" => EdgeKind::RefInit,
            "/kythe/edge/ref/init/implicit" => EdgeKind::RefInitImplicit,
            "/kythe/edge/ref/queries" => EdgeKind::RefQueries,
            "/kythe/edge/ref/writes" => EdgeKind::RefWrites,
            "/kythe/edge/ref/writes/implicit" => EdgeKind::RefWritesImplicit,
            "/kythe/edge/specializes" => EdgeKind::Specializes,
            "/kythe/edge/specializes/speculative" => EdgeKind::SpecializesSpeculative,
            "/kythe/edge/typed" => EdgeKind::Typed,
            "/kythe/edge/undefines" => EdgeKind::Undefines,
            _ => match value.strip_prefix("/kythe/edge/param.") {
                None => Err(ParseErr::UnknownEdgeKind(value.to_string()))?,
                Some(num) => EdgeKind::Param(num.parse::<u8>().map_err(|_| ParseErr::ExpectedInt)?),
            },
        })
    }
}

#[derive(Debug, Default)]
pub struct RawNode {
    code: Option<String>,
    complete: Option<String>,
    loc_end: Option<String>,
    loc_start: Option<String>,
    node_kind: Option<String>,
    param_default: Option<String>,
    subkind: Option<String>,
    tag_deprecated: Option<String>,
    tag_static: Option<String>,
    text: Option<String>,
}

const FACT_CODE: &'static str = "/kythe/code";
const FACT_COMPLETE: &'static str = "/kythe/complete";
const FACT_LOC_END: &'static str = "/kythe/loc/end";
const FACT_LOC_START: &'static str = "/kythe/loc/start";
const FACT_NODE_KIND: &'static str = "/kythe/node/kind";
const FACT_PARAM_DEFAULT: &'static str = "/kythe/param/default";
const FACT_SUBKIND: &'static str = "/kythe/subkind";
const FACT_TAG_DEPRECATED: &'static str = "/kythe/tag/deprecated";
const FACT_TAG_STATIC: &'static str = "/kythe/tag/static";
const FACT_TEXT: &'static str = "/kythe/text";

impl RawNode {
    fn get_mut(&mut self, fact_name: &str) -> Result<&mut Option<String>> {
        Ok(match fact_name {
            FACT_CODE => &mut self.code,
            FACT_COMPLETE => &mut self.complete,
            FACT_LOC_END => &mut self.loc_end,
            FACT_LOC_START => &mut self.loc_start,
            FACT_NODE_KIND => &mut self.node_kind,
            FACT_PARAM_DEFAULT => &mut self.param_default,
            FACT_SUBKIND => &mut self.subkind,
            FACT_TAG_DEPRECATED => &mut self.tag_deprecated,
            FACT_TAG_STATIC => &mut self.tag_static,
            FACT_TEXT => &mut self.text,
            _ => Err(ParseErr::UnknownFactName(fact_name.to_string()))?,
        })
    }

    fn set(&mut self, fact_name: &str, fact_value: String) -> Result<bool> {
        Ok(self.get_mut(fact_name)?.replace(fact_value).is_none())
    }

    #[allow(dead_code)]
    fn get_code(&self) -> Result<&String> {
        self.code.as_ref().ok_or(ParseErr::MissingFact(FACT_CODE))
    }

    fn get_complete(&self) -> Result<&String> {
        self.complete
            .as_ref()
            .ok_or(ParseErr::MissingFact(FACT_COMPLETE))
    }

    fn get_loc_end(&self) -> Result<usize> {
        self.loc_end
            .as_ref()
            .ok_or(ParseErr::MissingFact(FACT_LOC_END))?
            .parse::<usize>()
            .map_err(|_| ParseErr::ExpectedInt)
    }

    fn get_loc_start(&self) -> Result<usize> {
        self.loc_start
            .as_ref()
            .ok_or(ParseErr::MissingFact(FACT_LOC_START))?
            .parse::<usize>()
            .map_err(|_| ParseErr::ExpectedInt)
    }

    fn get_node_kind(&self) -> Result<&String> {
        self.node_kind
            .as_ref()
            .ok_or(ParseErr::MissingFact(FACT_NODE_KIND))
    }

    #[allow(dead_code)]
    fn get_param_default(&self) -> Result<&String> {
        self.param_default
            .as_ref()
            .ok_or(ParseErr::MissingFact(FACT_PARAM_DEFAULT))
    }

    fn get_subkind(&self) -> Result<&String> {
        self.subkind
            .as_ref()
            .ok_or(ParseErr::MissingFact(FACT_SUBKIND))
    }

    #[allow(dead_code)]
    fn get_tag_deprecated(&self) -> Result<&String> {
        self.tag_deprecated
            .as_ref()
            .ok_or(ParseErr::MissingFact(FACT_TAG_DEPRECATED))
    }

    #[allow(dead_code)]
    fn get_tag_static(&self) -> Result<&String> {
        self.tag_static
            .as_ref()
            .ok_or(ParseErr::MissingFact(FACT_TAG_STATIC))
    }

    fn to_text(self) -> Result<String> {
        self.text.ok_or(ParseErr::MissingFact(FACT_TEXT))
    }

    fn is_none(&self) -> bool {
        self.code.is_none()
            && self.complete.is_none()
            && self.loc_end.is_none()
            && self.loc_start.is_none()
            && self.node_kind.is_none()
            && self.param_default.is_none()
            && self.subkind.is_none()
            && self.tag_deprecated.is_none()
            && self.tag_static.is_none()
            && self.text.is_none()
    }
}

#[derive(Debug)]
pub struct Pos {
    start: usize,
    end: usize,
}

impl TryFrom<&RawNode> for Pos {
    type Error = ParseErr;

    fn try_from(value: &RawNode) -> Result<Self> {
        Ok(Pos {
            start: value.get_loc_start()?,
            end: value.get_loc_end()?,
        })
    }
}

#[derive(Debug)]
pub enum AnchorKind {
    Explicit(Pos),
    Implicit,
}

impl TryFrom<&RawNode> for AnchorKind {
    type Error = ParseErr;

    fn try_from(value: &RawNode) -> Result<Self> {
        Ok(match &value.subkind {
            None => AnchorKind::Explicit(Pos::try_from(value)?),
            Some(subkind) => match subkind.as_str() {
                "implicit" => AnchorKind::Implicit,
                _ => Err(ParseErr::UnknownAnchorKind(subkind.to_string()))?,
            },
        })
    }
}

#[derive(Debug)]
pub enum FunctionKind {
    Constructor,
    Destructor,
    None,
}

impl TryFrom<&str> for FunctionKind {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "constructor" => FunctionKind::Constructor,
            "initializer" => FunctionKind::Constructor,
            "destructor" => FunctionKind::Destructor,
            "none" => FunctionKind::None,
            _ => Err(ParseErr::UnknownFunctionKind(value.to_string()))?,
        })
    }
}

impl TryFrom<&RawNode> for FunctionKind {
    type Error = ParseErr;

    fn try_from(value: &RawNode) -> Result<Self> {
        Ok(match &value.subkind {
            Some(subkind) => FunctionKind::try_from(subkind.as_str())?,
            None => FunctionKind::None,
        })
    }
}

// C++ specific
#[derive(Debug)]
pub enum RecordKind {
    Class,
    Struct,
    Union,
}

impl TryFrom<&str> for RecordKind {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "class" => RecordKind::Class,
            "struct" => RecordKind::Struct,
            "union" => RecordKind::Union,
            _ => Err(ParseErr::UnknownRecordKind(value.to_string()))?,
        })
    }
}

impl TryFrom<&RawNode> for RecordKind {
    type Error = ParseErr;

    fn try_from(value: &RawNode) -> Result<Self> {
        Ok(RecordKind::try_from(value.get_subkind()?.as_str())?)
    }
}

// C++ specific
#[derive(Debug)]
pub enum SumKind {
    Enum,
    EnumClass,
}

impl TryFrom<&str> for SumKind {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "enum" => SumKind::Enum,
            "enumClass" => SumKind::EnumClass,
            _ => Err(ParseErr::UnknownSumKind(value.to_string()))?,
        })
    }
}

impl TryFrom<&RawNode> for SumKind {
    type Error = ParseErr;

    fn try_from(value: &RawNode) -> Result<Self> {
        Ok(SumKind::try_from(value.get_subkind()?.as_str())?)
    }
}

#[derive(Debug)]
pub enum VariableKind {
    Local,
    LocalParam,
    Field,
    Import,
    Unspecified,
}

impl TryFrom<&str> for VariableKind {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "local" => VariableKind::Local,
            "local/parameter" => VariableKind::LocalParam,
            "field" => VariableKind::Field,
            "import" => VariableKind::Import,
            _ => Err(ParseErr::UnknownVariableKind(value.to_string()))?,
        })
    }
}

impl TryFrom<&RawNode> for VariableKind {
    type Error = ParseErr;

    fn try_from(value: &RawNode) -> Result<Self> {
        Ok(match &value.subkind {
            Some(subkind) => VariableKind::try_from(subkind.as_str())?,
            None => VariableKind::Unspecified,
        })
    }
}

#[derive(Debug)]
pub enum CompleteStatus {
    Incomplete,
    Complete,
    Definition,
}

impl TryFrom<&str> for CompleteStatus {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "incomplete" => CompleteStatus::Incomplete,
            "complete" => CompleteStatus::Complete,
            "definition" => CompleteStatus::Definition,
            _ => Err(ParseErr::UnknownComplete(value.to_string()))?,
        })
    }
}

impl TryFrom<&RawNode> for CompleteStatus {
    type Error = ParseErr;

    fn try_from(value: &RawNode) -> Result<Self> {
        Ok(CompleteStatus::try_from(value.get_complete()?.as_str())?)
    }
}

#[derive(Debug)]
pub enum Node {
    Abs,
    Absvar,
    Anchor(AnchorKind),
    Constant(String),
    // Diagnostic(String),
    Doc(String),
    File(String),
    // Interface,
    Function(CompleteStatus, FunctionKind),
    Lookup(String),
    Macro,
    Meta,
    // Name,
    Package,
    // Process,
    Record(CompleteStatus, RecordKind),
    Sum(CompleteStatus, SumKind),
    // Symbol,
    Talias,
    Tapp,
    Tbuiltin,
    Tnominal,
    Tsigma,
    // Tvar,
    Variable(CompleteStatus, VariableKind),
    // Vcs,
    None, // Technically not allowed by spec but appears anyway.
}

impl TryFrom<RawNode> for Node {
    type Error = ParseErr;

    fn try_from(raw: RawNode) -> Result<Self> {
        if raw.is_none() {
            return Ok(Node::None);
        }

        let node_kind = raw.get_node_kind()?;

        let inner = match node_kind.as_str() {
            "abs" => Ok(Node::Abs),
            "absvar" => Ok(Node::Absvar),
            "anchor" => Ok(Node::Anchor(AnchorKind::try_from(&raw)?)),
            "constant" => Ok(Node::Constant(raw.to_text()?)),
            "doc" => Ok(Node::Doc(raw.to_text()?)),
            "file" => Ok(Node::File(raw.to_text()?)),
            "function" => Ok(Node::Function(
                CompleteStatus::try_from(&raw)?,
                FunctionKind::try_from(&raw)?,
            )),
            "lookup" => Ok(Node::Lookup(raw.to_text()?)),
            "macro" => Ok(Node::Macro),
            "meta" => Ok(Node::Meta),
            "package" => Ok(Node::Package),
            "record" => Ok(Node::Record(
                CompleteStatus::try_from(&raw)?,
                RecordKind::try_from(&raw)?,
            )),
            "sum" => Ok(Node::Sum(
                CompleteStatus::try_from(&raw)?,
                SumKind::try_from(&raw)?,
            )),
            "talias" => Ok(Node::Talias),
            "tapp" => Ok(Node::Tapp),
            "tbuiltin" => Ok(Node::Tbuiltin),
            "tnominal" => Ok(Node::Tnominal),
            "tsigma" => Ok(Node::Tsigma),
            "variable" => Ok(Node::Variable(
                CompleteStatus::try_from(&raw)?,
                VariableKind::try_from(&raw)?,
            )),
            _ => Err(ParseErr::UnknownNodeKind(node_kind.to_string())),
        };

        inner
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeIndex(usize);

impl From<&NodeIndex> for usize {
    fn from(value: &NodeIndex) -> Self {
        value.0
    }
}

#[derive(Debug, Default)]
pub struct RawKGraph {
    nodes: Vec<RawNode>,
    edges: KindedEdgeBag<EdgeKind, NodeIndex>,
    tickets: BiHashMap<Ticket, NodeIndex>,
}

impl RawKGraph {
    #[allow(dead_code)]
    pub fn open(_: &Path) -> Result<Self> {
        todo!()
    }

    fn reserve(&mut self, ticket: Ticket) -> NodeIndex {
        match self.tickets.get_by_left(&ticket) {
            Some(index) => *index,
            None => {
                let index = NodeIndex(self.nodes.len());
                self.nodes.push(RawNode::default());
                self.tickets.insert(ticket, index);
                index
            }
        }
    }

    fn put_fact(&mut self, index: NodeIndex, name: String, value: String) -> Result<bool> {
        self.nodes[index.0].set(&name, value)
    }

    fn put_edge(&mut self, kind: String, src: NodeIndex, tgt: NodeIndex) -> Result<usize> {
        Ok(self
            .edges
            .insert(EdgeKind::try_from(kind.as_str())?, src, tgt))
    }
}

impl TryFrom<EntryReader> for RawKGraph {
    type Error = ParseErr;

    fn try_from(reader: EntryReader) -> Result<Self> {
        let mut graph = RawKGraph::default();

        for entry in reader {
            match entry {
                Entry::Edge {
                    src,
                    tgt,
                    edge_kind,
                    ..
                } => {
                    let src_idx = graph.reserve(src);
                    let tgt_idx = graph.reserve(tgt);
                    graph.put_edge(edge_kind, src_idx, tgt_idx)?;
                }
                Entry::Node {
                    src,
                    fact_name,
                    fact_value,
                } => {
                    let idx = graph.reserve(src);
                    let fact_value = String::from_utf8_lossy(
                        &base64::decode(fact_value.unwrap_or_default()).unwrap(),
                    )
                    .to_string();
                    graph.put_fact(idx, fact_name, fact_value)?;
                }
            }
        }

        return Ok(graph);
    }
}

#[derive(Debug)]
pub struct NodeTriple<'a> {
    pub index: &'a NodeIndex,
    pub node: &'a Node,
    pub ticket: &'a Ticket,
}

pub struct KGraph {
    pub nodes: Vec<Node>,
    pub edges: KindedEdgeBag<EdgeKind, NodeIndex>,
    pub tickets: BiHashMap<Ticket, NodeIndex>,
}

impl KGraph {
    #[allow(dead_code)]
    pub fn open(_: &Path) -> Result<Self> {
        todo!()
    }

    pub fn triple<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeTriple> {
        let node = self.nodes.get(index.0)?;
        let ticket = self.tickets.get_by_right(index)?;
        Some(NodeTriple {
            index,
            node,
            ticket,
        })
    }

    pub fn parent_of(&self, index: &NodeIndex) -> Option<&NodeIndex> {
        self.edges.outgoing(&EdgeKind::Childof, index).exactly_one().ok().map(|(i, _)| i)
    }

    pub fn name_of(&self, triple: &NodeTriple) -> Option<&str> {
        if let Node::Anchor(AnchorKind::Explicit(pos)) = triple.node {
            return self.lookup_pos(&file_of(triple.ticket), pos);
        }

        for (index, _) in self.edges.incoming(&EdgeKind::DefinesBinding, &triple.index) {
            let name = self.name_of(&self.triple(index)?);

            if name.is_some() {
                return name;
            }
        }

        return None;
    }

    fn lookup_pos(&self, ticket: &Ticket, pos: &Pos) -> Option<&str> {
        let index = self.tickets.get_by_left(ticket)?;

        match self.nodes.get(index.0)? {
            Node::File(text) => Some(&text[pos.start..pos.end]),
            _ => None,
        }
    }
}

fn file_of(ticket: &Ticket) -> Ticket {
    Ticket {
        corpus: ticket.corpus.clone(),
        language: None,
        path: ticket.path.clone(),
        root: ticket.root.clone(),
        signature: None,
    }
}

impl TryFrom<RawKGraph> for KGraph {
    type Error = ParseErr;

    fn try_from(raw: RawKGraph) -> Result<Self> {
        let nodes = raw
            .nodes
            .into_iter()
            .enumerate()
            .map(|(i, raw_node)| {
                Node::try_from(raw_node)
                    .map_err(|e| ParseErr::SequencingErr(NodeIndex(i), Box::new(e)))
            })
            .collect::<Result<Vec<Node>>>()?;

        Ok(KGraph {
            nodes,
            edges: raw.edges,
            tickets: raw.tickets,
        })
    }
}
