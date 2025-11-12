use async_trait::async_trait;
use rig::tool::Tool;
use roma_core::Result;
use roma_storage::ExecutionStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::base::{build_error_response, build_success_response, Toolkit};

pub struct FileToolkit {
    storage: Arc<ExecutionStorage>,
}

impl FileToolkit {
    pub fn new(storage: Arc<ExecutionStorage>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl Toolkit for FileToolkit {
    fn name(&self) -> &str {
        "file"
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        vec![
            Arc::new(ReadFileTool::new(self.storage.clone())),
            Arc::new(WriteFileTool::new(self.storage.clone())),
            Arc::new(ListFilesTool::new(self.storage.clone())),
        ]
    }
}

#[derive(Clone)]
struct ReadFileTool {
    storage: Arc<ExecutionStorage>,
}

impl ReadFileTool {
    fn new(storage: Arc<ExecutionStorage>) -> Self {
        Self { storage }
    }
}

#[derive(Deserialize, Serialize)]
struct ReadFileArgs {
    filename: String,
}

#[async_trait]
impl Tool for ReadFileTool {
    const NAME: &'static str = "read_file";

    type Error = String;
    type Args = ReadFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::tool::ToolDefinition {
        rig::tool::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read the contents of a file from the execution storage".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": {
                        "type": "string",
                        "description": "The name of the file to read"
                    }
                },
                "required": ["filename"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        match self.storage.read_artifact(&args.filename).await {
            Ok(content) => Ok(build_success_response(serde_json::json!({ "content": content }))),
            Err(e) => Ok(build_error_response(&e.to_string())),
        }
    }
}

#[derive(Clone)]
struct WriteFileTool {
    storage: Arc<ExecutionStorage>,
}

impl WriteFileTool {
    fn new(storage: Arc<ExecutionStorage>) -> Self {
        Self { storage }
    }
}

#[derive(Deserialize, Serialize)]
struct WriteFileArgs {
    filename: String,
    content: String,
}

#[async_trait]
impl Tool for WriteFileTool {
    const NAME: &'static str = "write_file";

    type Error = String;
    type Args = WriteFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::tool::ToolDefinition {
        rig::tool::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Write content to a file in the execution storage".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": {
                        "type": "string",
                        "description": "The name of the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file"
                    }
                },
                "required": ["filename", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        match self.storage.write_artifact(&args.filename, &args.content).await {
            Ok(_) => Ok(build_success_response(
                serde_json::json!({ "message": "File written successfully" }),
            )),
            Err(e) => Ok(build_error_response(&e.to_string())),
        }
    }
}

#[derive(Clone)]
struct ListFilesTool {
    storage: Arc<ExecutionStorage>,
}

impl ListFilesTool {
    fn new(storage: Arc<ExecutionStorage>) -> Self {
        Self { storage }
    }
}

#[derive(Deserialize, Serialize)]
struct ListFilesArgs {}

#[async_trait]
impl Tool for ListFilesTool {
    const NAME: &'static str = "list_files";

    type Error = String;
    type Args = ListFilesArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::tool::ToolDefinition {
        rig::tool::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "List all files in the execution storage".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        match self.storage.list_artifacts().await {
            Ok(files) => Ok(build_success_response(serde_json::json!({ "files": files }))),
            Err(e) => Ok(build_error_response(&e.to_string())),
        }
    }
}
