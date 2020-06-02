//! Conversion of rust-analyzer specific types to lsp_types equivalents.
use ra_db::{FileId, FileRange};
use ra_ide::{
    Assist, CompletionItem, CompletionItemKind, Documentation, FileSystemEdit, Fold, FoldKind,
    FunctionSignature, Highlight, HighlightModifier, HighlightTag, HighlightedRange, Indel,
    InlayHint, InlayKind, InsertTextFormat, LineIndex, NavigationTarget, ReferenceAccess, Runnable,
    RunnableKind, Severity, SourceChange, SourceFileEdit, TextEdit,
};
use ra_syntax::{SyntaxKind, TextRange, TextSize};
use ra_vfs::LineEndings;

use crate::{
    cargo_target_spec::CargoTargetSpec, lsp_ext, semantic_tokens, world::WorldSnapshot, Result,
};

pub(crate) fn position(line_index: &LineIndex, offset: TextSize) -> lsp_types::Position {
    let line_col = line_index.line_col(offset);
    let line = u64::from(line_col.line);
    let character = u64::from(line_col.col_utf16);
    lsp_types::Position::new(line, character)
}

pub(crate) fn range(line_index: &LineIndex, range: TextRange) -> lsp_types::Range {
    let start = position(line_index, range.start());
    let end = position(line_index, range.end());
    lsp_types::Range::new(start, end)
}

pub(crate) fn symbol_kind(syntax_kind: SyntaxKind) -> lsp_types::SymbolKind {
    match syntax_kind {
        SyntaxKind::FN_DEF => lsp_types::SymbolKind::Function,
        SyntaxKind::STRUCT_DEF => lsp_types::SymbolKind::Struct,
        SyntaxKind::ENUM_DEF => lsp_types::SymbolKind::Enum,
        SyntaxKind::ENUM_VARIANT => lsp_types::SymbolKind::EnumMember,
        SyntaxKind::TRAIT_DEF => lsp_types::SymbolKind::Interface,
        SyntaxKind::MACRO_CALL => lsp_types::SymbolKind::Function,
        SyntaxKind::MODULE => lsp_types::SymbolKind::Module,
        SyntaxKind::TYPE_ALIAS_DEF => lsp_types::SymbolKind::TypeParameter,
        SyntaxKind::RECORD_FIELD_DEF => lsp_types::SymbolKind::Field,
        SyntaxKind::STATIC_DEF => lsp_types::SymbolKind::Constant,
        SyntaxKind::CONST_DEF => lsp_types::SymbolKind::Constant,
        SyntaxKind::IMPL_DEF => lsp_types::SymbolKind::Object,
        _ => lsp_types::SymbolKind::Variable,
    }
}

pub(crate) fn document_highlight_kind(
    reference_access: ReferenceAccess,
) -> lsp_types::DocumentHighlightKind {
    match reference_access {
        ReferenceAccess::Read => lsp_types::DocumentHighlightKind::Read,
        ReferenceAccess::Write => lsp_types::DocumentHighlightKind::Write,
    }
}

pub(crate) fn diagnostic_severity(severity: Severity) -> lsp_types::DiagnosticSeverity {
    match severity {
        Severity::Error => lsp_types::DiagnosticSeverity::Error,
        Severity::WeakWarning => lsp_types::DiagnosticSeverity::Hint,
    }
}

pub(crate) fn documentation(documentation: Documentation) -> lsp_types::Documentation {
    let value = crate::markdown::format_docs(documentation.as_str());
    let markup_content = lsp_types::MarkupContent { kind: lsp_types::MarkupKind::Markdown, value };
    lsp_types::Documentation::MarkupContent(markup_content)
}

pub(crate) fn insert_text_format(
    insert_text_format: InsertTextFormat,
) -> lsp_types::InsertTextFormat {
    match insert_text_format {
        InsertTextFormat::Snippet => lsp_types::InsertTextFormat::Snippet,
        InsertTextFormat::PlainText => lsp_types::InsertTextFormat::PlainText,
    }
}

pub(crate) fn completion_item_kind(
    completion_item_kind: CompletionItemKind,
) -> lsp_types::CompletionItemKind {
    match completion_item_kind {
        CompletionItemKind::Keyword => lsp_types::CompletionItemKind::Keyword,
        CompletionItemKind::Snippet => lsp_types::CompletionItemKind::Snippet,
        CompletionItemKind::Module => lsp_types::CompletionItemKind::Module,
        CompletionItemKind::Function => lsp_types::CompletionItemKind::Function,
        CompletionItemKind::Struct => lsp_types::CompletionItemKind::Struct,
        CompletionItemKind::Enum => lsp_types::CompletionItemKind::Enum,
        CompletionItemKind::EnumVariant => lsp_types::CompletionItemKind::EnumMember,
        CompletionItemKind::BuiltinType => lsp_types::CompletionItemKind::Struct,
        CompletionItemKind::Binding => lsp_types::CompletionItemKind::Variable,
        CompletionItemKind::Field => lsp_types::CompletionItemKind::Field,
        CompletionItemKind::Trait => lsp_types::CompletionItemKind::Interface,
        CompletionItemKind::TypeAlias => lsp_types::CompletionItemKind::Struct,
        CompletionItemKind::Const => lsp_types::CompletionItemKind::Constant,
        CompletionItemKind::Static => lsp_types::CompletionItemKind::Value,
        CompletionItemKind::Method => lsp_types::CompletionItemKind::Method,
        CompletionItemKind::TypeParam => lsp_types::CompletionItemKind::TypeParameter,
        CompletionItemKind::Macro => lsp_types::CompletionItemKind::Method,
        CompletionItemKind::Attribute => lsp_types::CompletionItemKind::EnumMember,
    }
}

pub(crate) fn text_edit(
    line_index: &LineIndex,
    line_endings: LineEndings,
    indel: Indel,
) -> lsp_types::TextEdit {
    let range = range(line_index, indel.delete);
    let new_text = match line_endings {
        LineEndings::Unix => indel.insert,
        LineEndings::Dos => indel.insert.replace('\n', "\r\n"),
    };
    lsp_types::TextEdit { range, new_text }
}

pub(crate) fn snippet_text_edit(
    line_index: &LineIndex,
    line_endings: LineEndings,
    is_snippet: bool,
    indel: Indel,
) -> lsp_ext::SnippetTextEdit {
    let text_edit = text_edit(line_index, line_endings, indel);
    let insert_text_format =
        if is_snippet { Some(lsp_types::InsertTextFormat::Snippet) } else { None };
    lsp_ext::SnippetTextEdit {
        range: text_edit.range,
        new_text: text_edit.new_text,
        insert_text_format,
    }
}

pub(crate) fn text_edit_vec(
    line_index: &LineIndex,
    line_endings: LineEndings,
    text_edit: TextEdit,
) -> Vec<lsp_types::TextEdit> {
    text_edit.into_iter().map(|indel| self::text_edit(line_index, line_endings, indel)).collect()
}

pub(crate) fn snippet_text_edit_vec(
    line_index: &LineIndex,
    line_endings: LineEndings,
    is_snippet: bool,
    text_edit: TextEdit,
) -> Vec<lsp_ext::SnippetTextEdit> {
    text_edit
        .into_iter()
        .map(|indel| self::snippet_text_edit(line_index, line_endings, is_snippet, indel))
        .collect()
}

pub(crate) fn completion_item(
    line_index: &LineIndex,
    line_endings: LineEndings,
    completion_item: CompletionItem,
) -> lsp_types::CompletionItem {
    let mut additional_text_edits = Vec::new();
    let mut text_edit = None;
    // LSP does not allow arbitrary edits in completion, so we have to do a
    // non-trivial mapping here.
    let source_range = completion_item.source_range();
    for indel in completion_item.text_edit().iter() {
        if indel.delete.contains_range(source_range) {
            text_edit = Some(if indel.delete == source_range {
                self::text_edit(line_index, line_endings, indel.clone())
            } else {
                assert!(source_range.end() == indel.delete.end());
                let range1 = TextRange::new(indel.delete.start(), source_range.start());
                let range2 = source_range;
                let indel1 = Indel::replace(range1, String::new());
                let indel2 = Indel::replace(range2, indel.insert.clone());
                additional_text_edits.push(self::text_edit(line_index, line_endings, indel1));
                self::text_edit(line_index, line_endings, indel2)
            })
        } else {
            assert!(source_range.intersect(indel.delete).is_none());
            let text_edit = self::text_edit(line_index, line_endings, indel.clone());
            additional_text_edits.push(text_edit);
        }
    }
    let text_edit = text_edit.unwrap();

    let mut res = lsp_types::CompletionItem {
        label: completion_item.label().to_string(),
        detail: completion_item.detail().map(|it| it.to_string()),
        filter_text: Some(completion_item.lookup().to_string()),
        kind: completion_item.kind().map(completion_item_kind),
        text_edit: Some(text_edit.into()),
        additional_text_edits: Some(additional_text_edits),
        documentation: completion_item.documentation().map(documentation),
        deprecated: Some(completion_item.deprecated()),
        ..Default::default()
    };

    if completion_item.score().is_some() {
        res.preselect = Some(true);
        // HACK: sort preselect items first
        res.sort_text = Some(format!(" {}", completion_item.label()));
    }

    if completion_item.deprecated() {
        res.tags = Some(vec![lsp_types::CompletionItemTag::Deprecated])
    }

    if completion_item.trigger_call_info() {
        res.command = Some(lsp_types::Command {
            title: "triggerParameterHints".into(),
            command: "editor.action.triggerParameterHints".into(),
            arguments: None,
        });
    }

    res.insert_text_format = Some(insert_text_format(completion_item.insert_text_format()));

    res
}

pub(crate) fn signature_information(
    signature: FunctionSignature,
    concise: bool,
) -> lsp_types::SignatureInformation {
    let (label, documentation, params) = if concise {
        let mut params = signature.parameters;
        if signature.has_self_param {
            params.remove(0);
        }
        (params.join(", "), None, params)
    } else {
        (signature.to_string(), signature.doc.map(documentation), signature.parameters)
    };

    let parameters: Vec<lsp_types::ParameterInformation> = params
        .into_iter()
        .map(|param| lsp_types::ParameterInformation {
            label: lsp_types::ParameterLabel::Simple(param),
            documentation: None,
        })
        .collect();

    lsp_types::SignatureInformation { label, documentation, parameters: Some(parameters) }
}

pub(crate) fn inlay_int(line_index: &LineIndex, inlay_hint: InlayHint) -> lsp_ext::InlayHint {
    lsp_ext::InlayHint {
        label: inlay_hint.label.to_string(),
        range: range(line_index, inlay_hint.range),
        kind: match inlay_hint.kind {
            InlayKind::ParameterHint => lsp_ext::InlayKind::ParameterHint,
            InlayKind::TypeHint => lsp_ext::InlayKind::TypeHint,
            InlayKind::ChainingHint => lsp_ext::InlayKind::ChainingHint,
        },
    }
}

pub(crate) fn semantic_tokens(
    text: &str,
    line_index: &LineIndex,
    highlights: Vec<HighlightedRange>,
) -> lsp_types::SemanticTokens {
    let mut builder = semantic_tokens::SemanticTokensBuilder::default();

    for highlight_range in highlights {
        let (type_, mods) = semantic_token_type_and_modifiers(highlight_range.highlight);
        let token_index = semantic_tokens::type_index(type_);
        let modifier_bitset = mods.0;

        for mut text_range in line_index.lines(highlight_range.range) {
            if text[text_range].ends_with('\n') {
                text_range =
                    TextRange::new(text_range.start(), text_range.end() - TextSize::of('\n'));
            }
            let range = range(&line_index, text_range);
            builder.push(range, token_index, modifier_bitset);
        }
    }

    builder.build()
}

fn semantic_token_type_and_modifiers(
    highlight: Highlight,
) -> (lsp_types::SemanticTokenType, semantic_tokens::ModifierSet) {
    let mut mods = semantic_tokens::ModifierSet::default();
    let type_ = match highlight.tag {
        HighlightTag::Struct => lsp_types::SemanticTokenType::STRUCT,
        HighlightTag::Enum => lsp_types::SemanticTokenType::ENUM,
        HighlightTag::Union => semantic_tokens::UNION,
        HighlightTag::TypeAlias => semantic_tokens::TYPE_ALIAS,
        HighlightTag::Trait => lsp_types::SemanticTokenType::INTERFACE,
        HighlightTag::BuiltinType => semantic_tokens::BUILTIN_TYPE,
        HighlightTag::SelfKeyword => semantic_tokens::SELF_KEYWORD,
        HighlightTag::SelfType => lsp_types::SemanticTokenType::TYPE,
        HighlightTag::Field => lsp_types::SemanticTokenType::PROPERTY,
        HighlightTag::Function => lsp_types::SemanticTokenType::FUNCTION,
        HighlightTag::Module => lsp_types::SemanticTokenType::NAMESPACE,
        HighlightTag::Constant => {
            mods |= semantic_tokens::CONSTANT;
            mods |= lsp_types::SemanticTokenModifier::STATIC;
            lsp_types::SemanticTokenType::VARIABLE
        }
        HighlightTag::Static => {
            mods |= lsp_types::SemanticTokenModifier::STATIC;
            lsp_types::SemanticTokenType::VARIABLE
        }
        HighlightTag::EnumVariant => semantic_tokens::ENUM_MEMBER,
        HighlightTag::Macro => lsp_types::SemanticTokenType::MACRO,
        HighlightTag::Local => lsp_types::SemanticTokenType::VARIABLE,
        HighlightTag::TypeParam => lsp_types::SemanticTokenType::TYPE_PARAMETER,
        HighlightTag::Lifetime => semantic_tokens::LIFETIME,
        HighlightTag::ByteLiteral | HighlightTag::NumericLiteral => {
            lsp_types::SemanticTokenType::NUMBER
        }
        HighlightTag::BoolLiteral => semantic_tokens::BOOLEAN,
        HighlightTag::CharLiteral | HighlightTag::StringLiteral => {
            lsp_types::SemanticTokenType::STRING
        }
        HighlightTag::Comment => lsp_types::SemanticTokenType::COMMENT,
        HighlightTag::Attribute => semantic_tokens::ATTRIBUTE,
        HighlightTag::Keyword => lsp_types::SemanticTokenType::KEYWORD,
        HighlightTag::UnresolvedReference => semantic_tokens::UNRESOLVED_REFERENCE,
        HighlightTag::FormatSpecifier => semantic_tokens::FORMAT_SPECIFIER,
        HighlightTag::Operator => lsp_types::SemanticTokenType::OPERATOR,
    };

    for modifier in highlight.modifiers.iter() {
        let modifier = match modifier {
            HighlightModifier::Attribute => semantic_tokens::ATTRIBUTE_MODIFIER,
            HighlightModifier::Definition => lsp_types::SemanticTokenModifier::DECLARATION,
            HighlightModifier::ControlFlow => semantic_tokens::CONTROL_FLOW,
            HighlightModifier::Mutable => semantic_tokens::MUTABLE,
            HighlightModifier::Unsafe => semantic_tokens::UNSAFE,
        };
        mods |= modifier;
    }

    (type_, mods)
}

pub(crate) fn folding_range(
    text: &str,
    line_index: &LineIndex,
    line_folding_only: bool,
    fold: Fold,
) -> lsp_types::FoldingRange {
    let kind = match fold.kind {
        FoldKind::Comment => Some(lsp_types::FoldingRangeKind::Comment),
        FoldKind::Imports => Some(lsp_types::FoldingRangeKind::Imports),
        FoldKind::Mods | FoldKind::Block => None,
    };

    let range = range(line_index, fold.range);

    if line_folding_only {
        // Clients with line_folding_only == true (such as VSCode) will fold the whole end line
        // even if it contains text not in the folding range. To prevent that we exclude
        // range.end.line from the folding region if there is more text after range.end
        // on the same line.
        let has_more_text_on_end_line = text[TextRange::new(fold.range.end(), TextSize::of(text))]
            .chars()
            .take_while(|it| *it != '\n')
            .any(|it| !it.is_whitespace());

        let end_line = if has_more_text_on_end_line {
            range.end.line.saturating_sub(1)
        } else {
            range.end.line
        };

        lsp_types::FoldingRange {
            start_line: range.start.line,
            start_character: None,
            end_line,
            end_character: None,
            kind,
        }
    } else {
        lsp_types::FoldingRange {
            start_line: range.start.line,
            start_character: Some(range.start.character),
            end_line: range.end.line,
            end_character: Some(range.end.character),
            kind,
        }
    }
}

pub(crate) fn url(world: &WorldSnapshot, file_id: FileId) -> Result<lsp_types::Url> {
    world.file_id_to_uri(file_id)
}

pub(crate) fn versioned_text_document_identifier(
    world: &WorldSnapshot,
    file_id: FileId,
    version: Option<i64>,
) -> Result<lsp_types::VersionedTextDocumentIdentifier> {
    let res = lsp_types::VersionedTextDocumentIdentifier { uri: url(world, file_id)?, version };
    Ok(res)
}

pub(crate) fn location(world: &WorldSnapshot, frange: FileRange) -> Result<lsp_types::Location> {
    let url = url(world, frange.file_id)?;
    let line_index = world.analysis().file_line_index(frange.file_id)?;
    let range = range(&line_index, frange.range);
    let loc = lsp_types::Location::new(url, range);
    Ok(loc)
}

pub(crate) fn location_link(
    world: &WorldSnapshot,
    src: Option<FileRange>,
    target: NavigationTarget,
) -> Result<lsp_types::LocationLink> {
    let origin_selection_range = match src {
        Some(src) => {
            let line_index = world.analysis().file_line_index(src.file_id)?;
            let range = range(&line_index, src.range);
            Some(range)
        }
        None => None,
    };
    let (target_uri, target_range, target_selection_range) = location_info(world, target)?;
    let res = lsp_types::LocationLink {
        origin_selection_range,
        target_uri,
        target_range,
        target_selection_range,
    };
    Ok(res)
}

fn location_info(
    world: &WorldSnapshot,
    target: NavigationTarget,
) -> Result<(lsp_types::Url, lsp_types::Range, lsp_types::Range)> {
    let line_index = world.analysis().file_line_index(target.file_id())?;

    let target_uri = url(world, target.file_id())?;
    let target_range = range(&line_index, target.full_range());
    let target_selection_range =
        target.focus_range().map(|it| range(&line_index, it)).unwrap_or(target_range);
    Ok((target_uri, target_range, target_selection_range))
}

pub(crate) fn goto_definition_response(
    world: &WorldSnapshot,
    src: Option<FileRange>,
    targets: Vec<NavigationTarget>,
) -> Result<lsp_types::GotoDefinitionResponse> {
    if world.config.client_caps.location_link {
        let links = targets
            .into_iter()
            .map(|nav| location_link(world, src, nav))
            .collect::<Result<Vec<_>>>()?;
        Ok(links.into())
    } else {
        let locations = targets
            .into_iter()
            .map(|nav| {
                location(
                    world,
                    FileRange {
                        file_id: nav.file_id(),
                        range: nav.focus_range().unwrap_or(nav.range()),
                    },
                )
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(locations.into())
    }
}

pub(crate) fn snippet_text_document_edit(
    world: &WorldSnapshot,
    is_snippet: bool,
    source_file_edit: SourceFileEdit,
) -> Result<lsp_ext::SnippetTextDocumentEdit> {
    let text_document = versioned_text_document_identifier(world, source_file_edit.file_id, None)?;
    let line_index = world.analysis().file_line_index(source_file_edit.file_id)?;
    let line_endings = world.file_line_endings(source_file_edit.file_id);
    let edits = source_file_edit
        .edit
        .into_iter()
        .map(|it| snippet_text_edit(&line_index, line_endings, is_snippet, it))
        .collect();
    Ok(lsp_ext::SnippetTextDocumentEdit { text_document, edits })
}

pub(crate) fn resource_op(
    world: &WorldSnapshot,
    file_system_edit: FileSystemEdit,
) -> Result<lsp_types::ResourceOp> {
    let res = match file_system_edit {
        FileSystemEdit::CreateFile { source_root, path } => {
            let uri = world.path_to_uri(source_root, &path)?;
            lsp_types::ResourceOp::Create(lsp_types::CreateFile { uri, options: None })
        }
        FileSystemEdit::MoveFile { src, dst_source_root, dst_path } => {
            let old_uri = world.file_id_to_uri(src)?;
            let new_uri = world.path_to_uri(dst_source_root, &dst_path)?;
            lsp_types::ResourceOp::Rename(lsp_types::RenameFile { old_uri, new_uri, options: None })
        }
    };
    Ok(res)
}

pub(crate) fn snippet_workspace_edit(
    world: &WorldSnapshot,
    source_change: SourceChange,
) -> Result<lsp_ext::SnippetWorkspaceEdit> {
    let mut document_changes: Vec<lsp_ext::SnippetDocumentChangeOperation> = Vec::new();
    for op in source_change.file_system_edits {
        let op = resource_op(&world, op)?;
        document_changes.push(lsp_ext::SnippetDocumentChangeOperation::Op(op));
    }
    for edit in source_change.source_file_edits {
        let edit = snippet_text_document_edit(&world, source_change.is_snippet, edit)?;
        document_changes.push(lsp_ext::SnippetDocumentChangeOperation::Edit(edit));
    }
    let workspace_edit =
        lsp_ext::SnippetWorkspaceEdit { changes: None, document_changes: Some(document_changes) };
    Ok(workspace_edit)
}

pub(crate) fn workspace_edit(
    world: &WorldSnapshot,
    source_change: SourceChange,
) -> Result<lsp_types::WorkspaceEdit> {
    assert!(!source_change.is_snippet);
    snippet_workspace_edit(world, source_change).map(|it| it.into())
}

impl From<lsp_ext::SnippetWorkspaceEdit> for lsp_types::WorkspaceEdit {
    fn from(snippet_workspace_edit: lsp_ext::SnippetWorkspaceEdit) -> lsp_types::WorkspaceEdit {
        lsp_types::WorkspaceEdit {
            changes: None,
            document_changes: snippet_workspace_edit.document_changes.map(|changes| {
                lsp_types::DocumentChanges::Operations(
                    changes
                        .into_iter()
                        .map(|change| match change {
                            lsp_ext::SnippetDocumentChangeOperation::Op(op) => {
                                lsp_types::DocumentChangeOperation::Op(op)
                            }
                            lsp_ext::SnippetDocumentChangeOperation::Edit(edit) => {
                                lsp_types::DocumentChangeOperation::Edit(
                                    lsp_types::TextDocumentEdit {
                                        text_document: edit.text_document,
                                        edits: edit
                                            .edits
                                            .into_iter()
                                            .map(|edit| lsp_types::TextEdit {
                                                range: edit.range,
                                                new_text: edit.new_text,
                                            })
                                            .collect(),
                                    },
                                )
                            }
                        })
                        .collect(),
                )
            }),
        }
    }
}

pub fn call_hierarchy_item(
    world: &WorldSnapshot,
    target: NavigationTarget,
) -> Result<lsp_types::CallHierarchyItem> {
    let name = target.name().to_string();
    let detail = target.description().map(|it| it.to_string());
    let kind = symbol_kind(target.kind());
    let (uri, range, selection_range) = location_info(world, target)?;
    Ok(lsp_types::CallHierarchyItem { name, kind, tags: None, detail, uri, range, selection_range })
}

#[cfg(test)]
mod tests {
    use test_utils::extract_ranges;

    use super::*;

    #[test]
    fn conv_fold_line_folding_only_fixup() {
        let text = r#"<fold>mod a;
mod b;
mod c;</fold>

fn main() <fold>{
    if cond <fold>{
        a::do_a();
    }</fold> else <fold>{
        b::do_b();
    }</fold>
}</fold>"#;

        let (ranges, text) = extract_ranges(text, "fold");
        assert_eq!(ranges.len(), 4);
        let folds = vec![
            Fold { range: ranges[0], kind: FoldKind::Mods },
            Fold { range: ranges[1], kind: FoldKind::Block },
            Fold { range: ranges[2], kind: FoldKind::Block },
            Fold { range: ranges[3], kind: FoldKind::Block },
        ];

        let line_index = LineIndex::new(&text);
        let converted: Vec<lsp_types::FoldingRange> =
            folds.into_iter().map(|it| folding_range(&text, &line_index, true, it)).collect();

        let expected_lines = [(0, 2), (4, 10), (5, 6), (7, 9)];
        assert_eq!(converted.len(), expected_lines.len());
        for (folding_range, (start_line, end_line)) in converted.iter().zip(expected_lines.iter()) {
            assert_eq!(folding_range.start_line, *start_line);
            assert_eq!(folding_range.start_character, None);
            assert_eq!(folding_range.end_line, *end_line);
            assert_eq!(folding_range.end_character, None);
        }
    }
}

pub(crate) fn code_action(world: &WorldSnapshot, assist: Assist) -> Result<lsp_ext::CodeAction> {
    let res = lsp_ext::CodeAction {
        title: assist.label,
        group: if world.config.client_caps.code_action_group { assist.group_label } else { None },
        kind: Some(String::new()),
        edit: Some(snippet_workspace_edit(world, assist.source_change)?),
        command: None,
    };
    Ok(res)
}

pub(crate) fn runnable(
    world: &WorldSnapshot,
    file_id: FileId,
    runnable: Runnable,
) -> Result<lsp_ext::Runnable> {
    let spec = CargoTargetSpec::for_file(world, file_id)?;
    let target = spec.as_ref().map(|s| s.target.clone());
    let (cargo_args, executable_args) =
        CargoTargetSpec::runnable_args(spec, &runnable.kind, &runnable.cfg_exprs)?;
    let label = match &runnable.kind {
        RunnableKind::Test { test_id, .. } => format!("test {}", test_id),
        RunnableKind::TestMod { path } => format!("test-mod {}", path),
        RunnableKind::Bench { test_id } => format!("bench {}", test_id),
        RunnableKind::DocTest { test_id, .. } => format!("doctest {}", test_id),
        RunnableKind::Bin => {
            target.map_or_else(|| "run binary".to_string(), |t| format!("run {}", t))
        }
    };
    let location = location_link(world, None, runnable.nav)?;

    Ok(lsp_ext::Runnable {
        label,
        location: Some(location),
        kind: lsp_ext::RunnableKind::Cargo,
        args: lsp_ext::CargoRunnable {
            workspace_root: world.workspace_root_for(file_id).map(|root| root.to_owned()),
            cargo_args,
            executable_args,
        },
    })
}
