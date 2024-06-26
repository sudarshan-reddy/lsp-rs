use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BaseMessage {
    pub jsonrpc: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMessage {
    #[serde(flatten)]
    pub base_message: BaseMessage,
    pub id: serde_json::Value,
    pub notification: u8,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMessage {
    #[serde(flatten)]
    pub base_message: BaseMessage,
    pub id: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotificationMessage {
    #[serde(flatten)]
    pub base_message: BaseMessage,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitializeParams {
    #[serde(rename = "processId")]
    pub process_id: u32,
    #[serde(rename = "rootUri")]
    pub root_uri: String,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
    pub capabilities: ClientCapabilities, // Direct embedding
    #[serde(rename = "workspaceFolders")]
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientCapabilities {
    pub workspace: Option<CapabilitiesWorkspace>, // Changed from HashMap to direct struct
    #[serde(rename = "textDocument")]
    pub text_document: Option<CapabilitiesTextDocument>, // Changed from HashMap to direct struct
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CapabilitiesWorkspace {
    #[serde(rename = "workspaceFolders")]
    pub workspace_folders: bool,
    #[serde(rename = "didChangeConfiguration")]
    pub did_change_configuration: DidChangeConfiguration,
    #[serde(rename = "workspaceEdit")]
    pub workspace_edit: WorkspaceEdit,
    pub configuration: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DidChangeConfiguration {
    #[serde(rename = "dynamicRegistration")]
    pub dynamic_registration: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceEdit {
    #[serde(rename = "documentChanges")]
    pub document_changes: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CapabilitiesTextDocument {
    pub hover: Hover,
    pub completion: Completion,
    #[serde(rename = "codeAction")]
    pub code_action: CodeAction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hover {
    #[serde(rename = "contentFormat")]
    pub content_format: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Completion {
    #[serde(rename = "completionItem")]
    pub completion_item: CompletionItem,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompletionItem {
    #[serde(rename = "snippetSupport")]
    pub snippet_support: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeAction {
    #[serde(rename = "codeActionLiteralSupport")]
    pub code_action_literal_support: CodeActionLiteralSupport,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeActionLiteralSupport {
    #[serde(rename = "codeActionKind")]
    pub code_action_kind: CodeActionKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeActionKind {
    #[serde(rename = "valueSet")]
    pub value_set: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    uri: String,
    range: Range,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Range {
    start: Position,
    end: Position,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    line: u32,
    character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Position { line, character }
    }
}

impl RequestMessage {
    /// Helper function to create a new `initialize` request message.
    /// id - The ID of the request message.
    /// process_id - The process ID of the client. (usually `std::process::id()`)
    /// root_uri - The root URI of the workspace. (e.g. `file://path/to/code`)
    /// client_name - The name of the client. (e.g. `vim-go`)
    /// workspace_folders - List of folders that the lsp needs context for.
    /// TODO: This function is currently a bit opinionated towards textdefintion.
    /// To have a custom initialize message, the workaround for now is to directly
    /// create a `RequestMessage` with desired capabilities.
    pub fn new_initialize(
        id: u32,
        process_id: u32,
        root_uri: String,
        client_name: String,
        client_version: String,
        workspace_folders: Vec<WorkspaceFolder>,
    ) -> Self {
        let client_info = ClientInfo {
            name: client_name,
            version: client_version,
        };

        let capabilities = ClientCapabilities {
            workspace: Some(CapabilitiesWorkspace {
                workspace_folders: true,
                did_change_configuration: DidChangeConfiguration {
                    dynamic_registration: true,
                },
                workspace_edit: WorkspaceEdit {
                    document_changes: true,
                },
                configuration: true,
            }),
            text_document: Some(CapabilitiesTextDocument {
                hover: Hover {
                    content_format: vec!["plaintext".to_string()],
                },
                completion: Completion {
                    completion_item: CompletionItem {
                        snippet_support: true,
                    },
                },
                code_action: CodeAction {
                    code_action_literal_support: CodeActionLiteralSupport {
                        code_action_kind: CodeActionKind {
                            value_set: vec![
                                "source.organizeImports".to_string(),
                                "refactor.rewrite".to_string(),
                                "refactor.extract".to_string(),
                            ],
                        },
                    },
                },
            }),
        };

        RequestMessage {
            base_message: BaseMessage {
                jsonrpc: "2.0".to_string(),
            },
            id: serde_json::Value::from(id),
            method: "initialize".to_string(),
            notification: 0,
            params: serde_json::to_value(InitializeParams {
                process_id,
                root_uri,
                client_info,
                capabilities,
                workspace_folders: Some(workspace_folders),
            })
            .unwrap(),
        }
    }

    /// Helper function to create a new `textDocument/definition` request message.
    /// id - The ID of the request message.
    /// uri - The URI of the text document. (e.g. `file://path/to/code/main.go`)
    /// line - The line number of the cursor position.
    /// character - The the cursor position of the character we want to get the definition of.
    pub fn new_get_definition(id: u32, uri: String, position: Position) -> Self {
        RequestMessage {
            base_message: BaseMessage {
                jsonrpc: "2.0".to_string(),
            },
            id: serde_json::Value::from(id),
            method: "textDocument/definition".to_string(),
            notification: 0,
            params: serde_json::json!({
                "textDocument": {
                    "uri": uri
                },
                "position": {
                    "line": position.line,
                    "character": position.character,
                }
            }),
        }
    }
}

impl NotificationMessage {
    /// Helper function to create a new `initialized` notification message.
    /// This message is sent by the client to the server once it has finished initializing
    /// and signals that the client is ready to receive requests.
    pub fn new_initialized() -> Self {
        NotificationMessage {
            base_message: BaseMessage {
                jsonrpc: "2.0".to_string(),
            },
            method: "initialized".to_string(),
            params: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

impl ResponseMessage {
    pub fn handle_initialize(&self) -> Result<()> {
        if self.error.is_some() {
            bail!("Error from LSP server: {:?}", self.error);
        };

        Ok(())
    }

    pub fn handle_definition(&self) -> Result<Vec<Location>> {
        if self.error.is_some() {
            bail!("Error from LSP server: {:?}", self.error);
        };

        if let Some(res) = &self.result {
            if res.is_null() {
                bail!("No definition found.");
            }
            let location: Result<Location, _> = serde_json::from_value(res.clone());
            let locations: Result<Vec<Location>, _> = serde_json::from_value(res.clone());

            match location {
                Ok(loc) => Ok(vec![loc]),
                Err(_) => match locations {
                    Ok(locs) => Ok(locs),
                    Err(_) => {
                        anyhow::bail!("Failed to parse definition location(s) from response.")
                    }
                },
            }
        } else {
            bail!("No definition found.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_initialize_message() {
        let process_id = std::process::id();
        let expected_init_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "notification": 0,
            "method": "initialize",
            "params": {
                "processId": process_id,
                "clientInfo": {
                    "name": "YourLSPClientName", // Customize this
                    "version": "1.0.0" // Optional: Adjust as necessary
                },
                "rootUri": "file://path/to/root",
                "capabilities": {
                    "workspace": {
                        "workspaceFolders": true,
                        "didChangeConfiguration": {
                            "dynamicRegistration": true
                        },
                        "workspaceEdit": {
                            "documentChanges": true
                        },
                        "configuration": true
                    },
                    "textDocument": {
                        "hover": {
                            "contentFormat": ["plaintext"]
                        },
                        "completion": {
                            "completionItem": {
                                "snippetSupport": true // Set to false if your client does not support snippets
                            }
                        },
                        "codeAction": {
                            "codeActionLiteralSupport": {
                                "codeActionKind": {
                                    "valueSet": ["source.organizeImports", "refactor.rewrite", "refactor.extract"]
                                }
                            }
                        }
                    }
                },
                "workspaceFolders": [{
                    "uri": "file://path/to/workspace",
                    "name": "file://path/to/workspace" ,
                }]
            }
        });

        let init_params = RequestMessage::new_initialize(
            1,
            process_id,
            "file://path/to/root".to_string(),
            "YourLSPClientName".to_string(),
            "1.0.0".to_string(),
            vec![WorkspaceFolder {
                uri: "file://path/to/workspace".to_string(),
                name: "file://path/to/workspace".to_string(),
            }],
        );

        // Check that the JSON serialization is correct
        let init_params_json = serde_json::to_value(init_params).unwrap();
        assert_eq!(expected_init_json, init_params_json);
    }

    #[test]
    fn test_initialized_notification() {
        let expected_initialized_json = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });

        let initialized_notification = NotificationMessage::new_initialized();
        let initialized_notification_json = serde_json::to_value(initialized_notification).unwrap();
        assert_eq!(expected_initialized_json, initialized_notification_json);
    }

    #[test]
    fn test_get_definition() {
        let expected_get_definition_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "notification": 0,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": "file://path/to/code/main.go"
                },
                "position": {
                    "line": 1,
                    "character": 2
                }
            }
        });

        let get_definition = RequestMessage::new_get_definition(
            1,
            "file://path/to/code/main.go".to_string(),
            Position {
                line: 1,
                character: 2,
            },
        );

        let get_definition_json = serde_json::to_value(get_definition).unwrap();
        assert_eq!(expected_get_definition_json, get_definition_json);
    }
}
