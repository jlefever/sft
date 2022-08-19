use std::collections::HashMap;
use std::hash::Hash;

use bimap::BiHashMap;
use itertools::Itertools;

use crate::collections::KindedEdgeBag;
use crate::io::{Entry, EntryReader, Ticket};

#[derive(Debug)]
pub enum TranslationErr {
    UnknownAnchorKind(String),
    UnknownEdgeKind(String),
    UnknownFactName(String),
    UnknownFunctionKind(String),
    UnknownRecordKind(Lang, String),
    UnknownSumKind(Lang, String),
    UnknownVariableKind(String),
    UnknownComplete(String),
    UnknownNodeKind(String),
    UnknownLang(String),
    MissingFact(&'static str),
    MissingLang,
    ExpectedInt,
    SequencingErr(NodeIndex, Box<TranslationErr>),
}

type Result<T> = std::result::Result<T, TranslationErr>;

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
    type Error = TranslationErr;

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
                None => Err(TranslationErr::UnknownEdgeKind(value.to_string()))?,
                Some(num) => {
                    EdgeKind::Param(num.parse::<u8>().map_err(|_| TranslationErr::ExpectedInt)?)
                }
            },
        })
    }
}

#[derive(Debug, Default)]
pub struct RawNodeValue {
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

impl RawNodeValue {
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
            _ => Err(TranslationErr::UnknownFactName(fact_name.to_string()))?,
        })
    }

    fn set(&mut self, fact_name: &str, fact_value: String) -> Result<bool> {
        Ok(self.get_mut(fact_name)?.replace(fact_value).is_none())
    }

    fn to_text(self) -> Result<String> {
        self.text.ok_or(TranslationErr::MissingFact(FACT_TEXT))
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

impl TryFrom<&RawNodeValue> for Pos {
    type Error = TranslationErr;

    fn try_from(value: &RawNodeValue) -> Result<Self> {
        let start = value
            .loc_start
            .as_deref()
            .ok_or(TranslationErr::MissingFact(FACT_LOC_START))?
            .parse::<usize>()
            .map_err(|_| TranslationErr::ExpectedInt)?;
        let end = value
            .loc_end
            .as_deref()
            .ok_or(TranslationErr::MissingFact(FACT_LOC_END))?
            .parse::<usize>()
            .map_err(|_| TranslationErr::ExpectedInt)?;

        Ok(Pos { start, end })
    }
}

#[derive(Debug)]
pub enum AnchorKind {
    Explicit(Pos),
    Implicit,
}

impl TryFrom<&RawNodeValue> for AnchorKind {
    type Error = TranslationErr;

    fn try_from(value: &RawNodeValue) -> Result<Self> {
        match &value.subkind.as_deref() {
            Some("implicit") => Ok(AnchorKind::Implicit),
            Some(str) => Err(TranslationErr::UnknownAnchorKind(str.to_string())),
            None => Ok(AnchorKind::Explicit(Pos::try_from(value)?)),
        }
    }
}

#[derive(Debug)]
pub enum CompleteStatus {
    Incomplete,
    Complete,
    Definition,
}

impl TryFrom<Option<&str>> for CompleteStatus {
    type Error = TranslationErr;

    fn try_from(value: Option<&str>) -> Result<Self> {
        match value {
            Some("incomplete") => Ok(CompleteStatus::Incomplete),
            Some("complete") => Ok(CompleteStatus::Complete),
            Some("definition") => Ok(CompleteStatus::Definition),
            Some(str) => Err(TranslationErr::UnknownComplete(str.to_string())),
            None => Err(TranslationErr::MissingFact(FACT_COMPLETE)),
        }
    }
}

#[derive(Debug)]
pub enum VariableKind {
    Local,
    LocalException,
    LocalParam,
    LocalResource,
    Field,
    Import,
    Unspecified,
}

impl TryFrom<Option<&str>> for VariableKind {
    type Error = TranslationErr;

    fn try_from(value: Option<&str>) -> Result<Self> {
        match value {
            Some("local") => Ok(VariableKind::Local),
            Some("local/exception") => Ok(VariableKind::LocalException),
            Some("local/parameter") => Ok(VariableKind::LocalParam),
            Some("local/resource") => Ok(VariableKind::LocalResource),
            Some("field") => Ok(VariableKind::Field),
            Some("import") => Ok(VariableKind::Import),
            Some(str) => Err(TranslationErr::UnknownVariableKind(str.to_string())),
            None => Ok(VariableKind::Unspecified),
        }
    }
}

#[derive(Debug)]
pub enum FunctionKind {
    Constructor,
    Destructor,
    Unspecified,
}

impl TryFrom<Option<&str>> for FunctionKind {
    type Error = TranslationErr;

    fn try_from(value: Option<&str>) -> Result<Self> {
        match value {
            Some("constructor") => Ok(FunctionKind::Constructor),
            Some("initializer") => Ok(FunctionKind::Constructor),
            Some("destructor") => Ok(FunctionKind::Destructor),
            Some("none") => Ok(FunctionKind::Unspecified),
            Some(str) => Err(TranslationErr::UnknownFunctionKind(str.to_string())),
            None => Ok(FunctionKind::Unspecified),
        }
    }
}

#[derive(Debug)]
pub enum Lang {
    Cpp,
    Java,
    Unspecified,
}

impl TryFrom<Option<&str>> for Lang {
    type Error = TranslationErr;

    fn try_from(value: Option<&str>) -> Result<Self> {
        match value {
            Some("c++") => Ok(Lang::Cpp),
            Some("java") => Ok(Lang::Java),
            Some(str) => Err(TranslationErr::UnknownLang(str.to_string())),
            None => Ok(Lang::Unspecified),
        }
    }
}

#[derive(Debug)]
pub enum RecordKind {
    Cpp(CppRecordKind),
    Java(JavaRecordKind),
}

#[derive(Debug)]
pub enum CppRecordKind {
    Class,
    Struct,
    Union,
}

impl TryFrom<Option<&str>> for CppRecordKind {
    type Error = TranslationErr;

    fn try_from(value: Option<&str>) -> Result<Self> {
        match value {
            Some("class") => Ok(CppRecordKind::Class),
            Some("struct") => Ok(CppRecordKind::Struct),
            Some("union") => Ok(CppRecordKind::Union),
            Some(str) => Err(TranslationErr::UnknownRecordKind(Lang::Cpp, str.to_string()))?,
            None => Err(TranslationErr::MissingFact(FACT_SUBKIND)),
        }
    }
}

#[derive(Debug)]
pub enum JavaRecordKind {
    Class,
}

impl TryFrom<Option<&str>> for JavaRecordKind {
    type Error = TranslationErr;

    fn try_from(value: Option<&str>) -> Result<Self> {
        match value {
            Some("class") => Ok(JavaRecordKind::Class),
            Some(str) => Err(TranslationErr::UnknownRecordKind(Lang::Java, str.to_string()))?,
            None => Err(TranslationErr::MissingFact(FACT_SUBKIND)),
        }
    }
}

impl TryFrom<(Option<&str>, &Lang)> for RecordKind {
    type Error = TranslationErr;

    fn try_from((value, lang): (Option<&str>, &Lang)) -> Result<Self> {
        match lang {
            Lang::Cpp => Ok(RecordKind::Cpp(CppRecordKind::try_from(value)?)),
            Lang::Java => Ok(RecordKind::Java(JavaRecordKind::try_from(value)?)),
            Lang::Unspecified => Err(TranslationErr::MissingLang),
        }
    }
}

#[derive(Debug)]
pub enum SumKind {
    Cpp(CppSumKind),
    Java(JavaSumKind),
}

#[derive(Debug)]
pub enum CppSumKind {
    Enum,
    EnumClass,
}

impl TryFrom<Option<&str>> for CppSumKind {
    type Error = TranslationErr;

    fn try_from(value: Option<&str>) -> Result<Self> {
        match value {
            Some("enum") => Ok(CppSumKind::Enum),
            Some("enumClass") => Ok(CppSumKind::EnumClass),
            Some(str) => Err(TranslationErr::UnknownSumKind(Lang::Cpp, str.to_string())),
            None => Err(TranslationErr::MissingFact(FACT_SUBKIND)),
        }
    }
}

#[derive(Debug)]
pub enum JavaSumKind {
    Enum,
}

impl TryFrom<Option<&str>> for JavaSumKind {
    type Error = TranslationErr;

    fn try_from(value: Option<&str>) -> Result<Self> {
        match value {
            Some("enum") => Ok(JavaSumKind::Enum),
            Some(str) => Err(TranslationErr::UnknownSumKind(Lang::Java, str.to_string())),
            None => Err(TranslationErr::MissingFact(FACT_SUBKIND)),
        }
    }
}

impl TryFrom<(Option<&str>, &Lang)> for SumKind {
    type Error = TranslationErr;

    fn try_from((value, lang): (Option<&str>, &Lang)) -> Result<Self> {
        match lang {
            Lang::Cpp => Ok(SumKind::Cpp(CppSumKind::try_from(value)?)),
            Lang::Java => Ok(SumKind::Java(JavaSumKind::try_from(value)?)),
            Lang::Unspecified => Err(TranslationErr::MissingLang)?,
        }
    }
}

#[derive(Debug)]
pub enum NodeKind {
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

impl TryFrom<(RawNodeValue, &Lang)> for NodeKind {
    type Error = TranslationErr;

    fn try_from((value, lang): (RawNodeValue, &Lang)) -> Result<Self> {
        if value.is_none() {
            return Ok(NodeKind::None);
        }

        match value.node_kind.as_deref() {
            Some("abs") => Ok(NodeKind::Abs),
            Some("absvar") => Ok(NodeKind::Absvar),
            Some("anchor") => Ok(NodeKind::Anchor(AnchorKind::try_from(&value)?)),
            Some("constant") => Ok(NodeKind::Constant(value.to_text()?)),
            Some("doc") => Ok(NodeKind::Doc(value.to_text()?)),
            Some("file") => Ok(NodeKind::File(value.to_text()?)),
            Some("function") => Ok(NodeKind::Function(
                CompleteStatus::try_from(value.complete.as_deref())?,
                FunctionKind::try_from(value.subkind.as_deref())?,
            )),
            Some("lookup") => Ok(NodeKind::Lookup(value.to_text()?)),
            Some("macro") => Ok(NodeKind::Macro),
            Some("meta") => Ok(NodeKind::Meta),
            Some("package") => Ok(NodeKind::Package),
            Some("record") => Ok(NodeKind::Record(
                CompleteStatus::try_from(value.complete.as_deref())?,
                RecordKind::try_from((value.subkind.as_deref(), lang))?,
            )),
            Some("sum") => Ok(NodeKind::Sum(
                CompleteStatus::try_from(value.complete.as_deref())?,
                SumKind::try_from((value.subkind.as_deref(), lang))?,
            )),
            Some("talias") => Ok(NodeKind::Talias),
            Some("tapp") => Ok(NodeKind::Tapp),
            Some("tbuiltin") => Ok(NodeKind::Tbuiltin),
            Some("tnominal") => Ok(NodeKind::Tnominal),
            Some("tsigma") => Ok(NodeKind::Tsigma),
            Some("variable") => Ok(NodeKind::Variable(
                CompleteStatus::try_from(value.complete.as_deref())?,
                VariableKind::try_from(value.subkind.as_deref())?,
            )),
            Some(str) => Err(TranslationErr::UnknownNodeKind(str.to_string())),
            None => Err(TranslationErr::MissingFact(FACT_NODE_KIND)),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct FileKey {
    pub corpus: Option<String>,
    pub path: Option<String>,
    pub root: Option<String>,
}

impl From<&Ticket> for FileKey {
    fn from(ticket: &Ticket) -> Self {
        FileKey {
            corpus: ticket.corpus.clone(),
            path: ticket.path.clone(),
            root: ticket.root.clone(),
        }
    }
}

pub struct Node {
    pub index: NodeIndex,
    pub signature: Option<String>,
    pub lang: Lang,
    pub file_key: FileKey,
    pub kind: NodeKind,
}

impl TryFrom<(NodeIndex, RawNodeValue, &Ticket)> for Node {
    type Error = TranslationErr;

    fn try_from((index, raw, ticket): (NodeIndex, RawNodeValue, &Ticket)) -> Result<Self> {
        let signature = ticket.signature.clone();
        let lang = Lang::try_from(ticket.language.as_deref())?;
        let file_key = FileKey::from(ticket);
        let kind = NodeKind::try_from((raw, &lang))?;

        Ok(Node { index, signature, lang, file_key, kind })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeIndex(pub usize);

#[derive(Debug, Default)]
pub struct RawGraph {
    nodes: Vec<RawNodeValue>,
    edges: KindedEdgeBag<EdgeKind, NodeIndex>,
    tickets: BiHashMap<Ticket, NodeIndex>,
}

impl RawGraph {
    fn reserve(&mut self, ticket: Ticket) -> NodeIndex {
        match self.tickets.get_by_left(&ticket) {
            Some(index) => *index,
            None => {
                let index = NodeIndex(self.nodes.len());
                self.nodes.push(RawNodeValue::default());
                self.tickets.insert(ticket, index);
                index
            }
        }
    }

    fn put_fact(&mut self, index: NodeIndex, name: String, value: String) -> Result<bool> {
        self.nodes[index.0].set(&name, value)
    }

    fn put_edge(&mut self, kind: String, src: NodeIndex, tgt: NodeIndex) -> Result<usize> {
        Ok(self.edges.insert(EdgeKind::try_from(kind.as_str())?, src, tgt))
    }
}

impl TryFrom<EntryReader> for RawGraph {
    type Error = TranslationErr;

    fn try_from(reader: EntryReader) -> Result<Self> {
        let mut graph = RawGraph::default();

        for entry in reader {
            match entry {
                Entry::Edge { src, tgt, edge_kind, .. } => {
                    let src_idx = graph.reserve(src);
                    let tgt_idx = graph.reserve(tgt);
                    graph.put_edge(edge_kind, src_idx, tgt_idx)?;
                }
                Entry::Node { src, fact_name, fact_value } => {
                    let idx = graph.reserve(src);
                    let decoded = base64::decode(fact_value.unwrap_or_default()).unwrap();
                    let fact_value = String::from_utf8_lossy(&decoded).to_string();
                    graph.put_fact(idx, fact_name, fact_value)?;
                }
            }
        }

        Ok(graph)
    }
}

pub struct SpecGraph {
    nodes: Vec<Node>,
    files: HashMap<FileKey, NodeIndex>,
    edges: KindedEdgeBag<EdgeKind, NodeIndex>,
}

impl SpecGraph {
    pub fn get_node(&self, index: &NodeIndex) -> Option<&Node> {
        self.nodes.get(index.0)
    }

    #[allow(dead_code)]
    pub fn get_file_bi(&self, index: &NodeIndex) -> Option<&NodeIndex> {
        self.get_file_bn(self.get_node(index)?)
    }

    pub fn get_file_bn(&self, node: &Node) -> Option<&NodeIndex> {
        self.files.get(&node.file_key)
    }

    pub fn get_name_bi(&self, index: &NodeIndex) -> Option<&str> {
        self.get_name_bn(self.get_node(index)?)
    }

    pub fn get_name_bn(&self, node: &Node) -> Option<&str> {
        if let NodeKind::Anchor(AnchorKind::Explicit(pos)) = &node.kind {
            return self.get_text_bi(self.get_file_bn(node)?, pos);
        }

        self.get_name_bi(self.get_binding(&node.index)?)
    }

    pub fn get_text_bi(&self, index: &NodeIndex, pos: &Pos) -> Option<&str> {
        self.get_text_bn(self.get_node(index)?, pos)
    }

    pub fn get_text_bn<'g, 'n>(&'g self, node: &'n Node, pos: &Pos) -> Option<&'n str> {
        match &node.kind {
            NodeKind::File(text) => Some(&text[pos.start..pos.end]),
            _ => None,
        }
    }

    pub fn get_binding(&self, index: &NodeIndex) -> Option<&NodeIndex> {
        let incoming = self.edges.incoming(&EdgeKind::DefinesBinding, &index);
        incoming.at_most_one().ok().unwrap().map(|(i, _)| i)
    }

    pub fn get_parent(&self, index: &NodeIndex) -> Option<&NodeIndex> {
        let incoming = self.edges.outgoing(&EdgeKind::Childof, &index);
        incoming.at_most_one().ok().unwrap().map(|(i, _)| i)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&EdgeKind, &NodeIndex, &NodeIndex, &usize)> + '_ {
        self.edges.iter()
    }
}

impl TryFrom<RawGraph> for SpecGraph {
    type Error = TranslationErr;

    fn try_from(raw_graph: RawGraph) -> Result<Self> {
        let edges = raw_graph.edges;
        let mut nodes = Vec::with_capacity(raw_graph.nodes.len());
        let mut files = HashMap::new();

        for (i, raw_node) in raw_graph.nodes.into_iter().enumerate() {
            let index = NodeIndex(i);
            let ticket = raw_graph.tickets.get_by_right(&index).unwrap();
            let node = Node::try_from((index, raw_node, ticket))
                .map_err(|e| TranslationErr::SequencingErr(index, Box::new(e)))?;

            if let NodeKind::File(_) = node.kind {
                files.insert(node.file_key.clone(), index);
            }

            nodes.push(node);
        }

        Ok(SpecGraph { nodes, files, edges })
    }
}
