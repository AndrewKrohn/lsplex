use std::{collections::HashMap, sync::LazyLock};
pub static CAP_MAP: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut cmap = HashMap::new();

    // ----- Lifecycle ---------------------------------------------------------
    cmap.insert("initialize", "initializeProvider");
    cmap.insert("initialized", "initializedProvider");
    cmap.insert("shutdown", "shutdownProvider");
    cmap.insert("exit", "exitProvider");

    // ----- Workspace ----------------------------------------------------------
    cmap.insert("workspace/didChangeConfiguration", "configChangeProvider");
    cmap.insert("workspace/didChangeWatchedFiles", "fileWatchProvider");
    cmap.insert("workspace/symbol", "workspaceSymbolProvider");
    cmap.insert("workspace/executeCommand", "executeCommandProvider");
    cmap.insert("workspace/workspaceFolders", "workspaceFolderProvider");
    cmap.insert("workspace/configuration", "workspaceConfigurationProvider");

    // ----- Text Document lifecycle -------------------------------------------
    cmap.insert("textDocument/didOpen", "didOpenProvider");
    cmap.insert("textDocument/didChange", "didChangeProvider");
    cmap.insert("textDocument/didClose", "didCloseProvider");
    cmap.insert("textDocument/didSave", "didSaveProvider");

    // ----- Navigation --------------------------------------------------------
    cmap.insert("textDocument/definition", "definitionProvider");
    cmap.insert("textDocument/typeDefinition", "typeDefinitionProvider");
    cmap.insert("textDocument/declaration", "declarationProvider");
    cmap.insert("textDocument/implementation", "implementationProvider");
    cmap.insert("textDocument/references", "referencesProvider");

    // ----- Hover & Signature -------------------------------------------------
    cmap.insert("textDocument/hover", "hoverProvider");
    cmap.insert("textDocument/signatureHelp", "signatureHelpProvider");

    // ----- Completion --------------------------------------------------------
    cmap.insert("textDocument/completion", "completionProvider");
    cmap.insert("completionItem/resolve", "completionResolveProvider");

    // ----- Document symbols --------------------------------------------------
    cmap.insert("textDocument/documentSymbol", "documentSymbolProvider");

    // ----- Code actions ------------------------------------------------------
    cmap.insert("textDocument/codeAction", "codeActionProvider");
    cmap.insert("codeAction/resolve", "codeActionResolveProvider");

    // ----- Code lens ---------------------------------------------------------
    cmap.insert("textDocument/codeLens", "codeLensProvider");
    cmap.insert("codeLens/resolve", "codeLensResolveProvider");

    // ----- Document links ----------------------------------------------------
    cmap.insert("textDocument/documentLink", "documentLinkProvider");
    cmap.insert("documentLink/resolve", "documentLinkResolveProvider");

    // ----- Highlighting ------------------------------------------------------
    cmap.insert(
        "textDocument/documentHighlight",
        "documentHighlightProvider",
    );

    // ----- Formatting --------------------------------------------------------
    cmap.insert("textDocument/formatting", "formattingProvider");
    cmap.insert("textDocument/rangeFormatting", "rangeFormattingProvider");
    cmap.insert("textDocument/onTypeFormatting", "onTypeFormattingProvider");

    // ----- Rename ------------------------------------------------------------
    cmap.insert("textDocument/rename", "renameProvider");

    // ----- Folding & Selection -----------------------------------------------
    cmap.insert("textDocument/foldingRange", "foldingRangeProvider");
    cmap.insert("textDocument/selectionRange", "selectionRangeProvider");

    // ----- Semantic tokens ---------------------------------------------------
    cmap.insert(
        "textDocument/semanticTokens/full",
        "semanticTokensFullProvider",
    );
    cmap.insert(
        "textDocument/semanticTokens/range",
        "semanticTokensRangeProvider",
    );
    cmap.insert(
        "textDocument/semanticTokens/full/delta",
        "semanticTokensDeltaProvider",
    );

    // ----- Inlay hints -------------------------------------------------------
    cmap.insert("textDocument/inlayHint", "inlayHintProvider");
    cmap.insert("inlayHint/resolve", "inlayHintResolveProvider");

    // ----- Window / UI -------------------------------------------------------
    cmap.insert("window/showMessageRequest", "showMessageRequestProvider");
    cmap.insert("window/logMessage", "logMessageProvider");
    cmap.insert("window/showDocument", "showDocumentProvider");
    cmap
});
