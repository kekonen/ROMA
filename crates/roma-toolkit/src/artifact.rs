use async_trait::async_trait;
use rig::tool::Tool;
use roma_core::{Artifact, ArtifactType, Result};
use roma_storage::ExecutionStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::base::{build_error_response, build_success_response, Toolkit};

pub struct ArtifactToolkit {
    storage: Arc<ExecutionStorage>,
}

impl ArtifactToolkit {
    pub fn new(storage: Arc<ExecutionStorage>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl Toolkit for ArtifactToolkit {
    fn name(&self) -> &str {
        "artifact"
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        vec![
            Arc::new(CreateArtifactTool::new(self.storage.clone())),
            Arc::new(GetArtifactTool::new(self.storage.clone())),
        ]
    }
}

#[derive(Clone)]
struct CreateArtifactTool {
    storage: Arc<ExecutionStorage>,
}

impl CreateArtifactTool {
    fn new(storage: Arc<ExecutionStorage>) -> Self {
        Self { storage }
    }
}

#[derive(Deserialize, Serialize)]
struct CreateArtifactArgs {
    artifact_type: String,
    name: String,
    content: String,
}

#[async_trait]
impl Tool for CreateArtifactTool {
    const NAME: &'static str = "create_artifact";

    type Error = String;
    type Args = CreateArtifactArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::tool::ToolDefinition {
        rig::tool::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Create and store an artifact (code, document, data, image, or other)".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "artifact_type": {
                        "type": "string",
                        "enum": ["code", "document", "data", "image", "other"],
                        "description": "Type of artifact to create"
                    },
                    "name": {
                        "type": "string",
                        "description": "Name of the artifact"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content of the artifact"
                    }
                },
                "required": ["artifact_type", "name", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        let artifact_type = match args.artifact_type.to_lowercase().as_str() {
            "code" => ArtifactType::Code,
            "document" => ArtifactType::Document,
            "data" => ArtifactType::Data,
            "image" => ArtifactType::Image,
            _ => ArtifactType::Other,
        };

        let artifact = Artifact::new(artifact_type, args.name.clone(), args.content);

        match self
            .storage
            .write_artifact(&format!("{}.json", args.name), &serde_json::to_string(&artifact).unwrap())
            .await
        {
            Ok(_) => Ok(build_success_response(serde_json::json!({
                "artifact_id": artifact.artifact_id,
                "message": "Artifact created successfully"
            }))),
            Err(e) => Ok(build_error_response(&e.to_string())),
        }
    }
}

#[derive(Clone)]
struct GetArtifactTool {
    storage: Arc<ExecutionStorage>,
}

impl GetArtifactTool {
    fn new(storage: Arc<ExecutionStorage>) -> Self {
        Self { storage }
    }
}

#[derive(Deserialize, Serialize)]
struct GetArtifactArgs {
    name: String,
}

#[async_trait]
impl Tool for GetArtifactTool {
    const NAME: &'static str = "get_artifact";

    type Error = String;
    type Args = GetArtifactArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::tool::ToolDefinition {
        rig::tool::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Retrieve a previously created artifact by name".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the artifact to retrieve"
                    }
                },
                "required": ["name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        match self
            .storage
            .read_artifact(&format!("{}.json", args.name))
            .await
        {
            Ok(content) => {
                let artifact: Artifact = serde_json::from_str(&content).unwrap_or_else(|_| {
                    Artifact::new(ArtifactType::Other, args.name.clone(), content)
                });
                Ok(build_success_response(serde_json::json!(artifact)))
            }
            Err(e) => Ok(build_error_response(&e.to_string())),
        }
    }
}
