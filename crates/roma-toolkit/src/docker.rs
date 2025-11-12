use async_trait::async_trait;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, WaitContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use futures::StreamExt;
use rig::tool::Tool;
use roma_core::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::base::{build_error_response, build_success_response, Toolkit};

pub struct DockerToolkit {
    docker: Docker,
}

impl DockerToolkit {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| roma_core::RomaError::ToolkitError(format!("Failed to connect to Docker: {}", e)))?;

        Ok(Self { docker })
    }
}

#[async_trait]
impl Toolkit for DockerToolkit {
    fn name(&self) -> &str {
        "docker"
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        vec![
            Arc::new(RunPythonCodeTool::new(self.docker.clone())),
            Arc::new(RunCommandTool::new(self.docker.clone())),
        ]
    }
}

#[derive(Clone)]
struct RunPythonCodeTool {
    docker: Docker,
}

impl RunPythonCodeTool {
    fn new(docker: Docker) -> Self {
        Self { docker }
    }
}

#[derive(Deserialize, Serialize)]
struct RunPythonCodeArgs {
    code: String,
}

#[async_trait]
impl Tool for RunPythonCodeTool {
    const NAME: &'static str = "run_python_code";

    type Error = String;
    type Args = RunPythonCodeArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::tool::ToolDefinition {
        rig::tool::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Execute Python code in a sandboxed Docker container".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Python code to execute"
                    }
                },
                "required": ["code"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        let container_name = format!("roma-python-{}", uuid::Uuid::new_v4());

        let config = Config {
            image: Some("python:3.11-slim"),
            cmd: Some(vec!["python", "-c", &args.code]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let container = match self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: &container_name,
                    ..Default::default()
                }),
                config,
            )
            .await
        {
            Ok(c) => c,
            Err(e) => return Ok(build_error_response(&format!("Failed to create container: {}", e))),
        };

        if let Err(e) = self
            .docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
        {
            return Ok(build_error_response(&format!("Failed to start container: {}", e)));
        }

        let wait_options = Some(WaitContainerOptions {
            condition: "not-running",
        });

        let mut wait_stream = self.docker.wait_container(&container.id, wait_options);

        while let Some(_) = wait_stream.next().await {}

        let logs = match self.docker.logs::<String>(
            &container.id,
            Some(bollard::container::LogsOptions {
                stdout: true,
                stderr: true,
                ..Default::default()
            }),
        ).try_collect::<Vec<_>>().await {
            Ok(logs) => logs.into_iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n"),
            Err(e) => format!("Failed to get logs: {}", e),
        };

        let _ = self.docker.remove_container(&container.id, None).await;

        Ok(build_success_response(serde_json::json!({ "output": logs })))
    }
}

#[derive(Clone)]
struct RunCommandTool {
    docker: Docker,
}

impl RunCommandTool {
    fn new(docker: Docker) -> Self {
        Self { docker }
    }
}

#[derive(Deserialize, Serialize)]
struct RunCommandArgs {
    command: String,
    image: Option<String>,
}

#[async_trait]
impl Tool for RunCommandTool {
    const NAME: &'static str = "run_command";

    type Error = String;
    type Args = RunCommandArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::tool::ToolDefinition {
        rig::tool::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Execute a shell command in a Docker container".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    },
                    "image": {
                        "type": "string",
                        "description": "Docker image to use (default: ubuntu:latest)"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        let image = args.image.unwrap_or_else(|| "ubuntu:latest".to_string());
        let container_name = format!("roma-cmd-{}", uuid::Uuid::new_v4());

        let config = Config {
            image: Some(image.as_str()),
            cmd: Some(vec!["sh", "-c", &args.command]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let container = match self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: &container_name,
                    ..Default::default()
                }),
                config,
            )
            .await
        {
            Ok(c) => c,
            Err(e) => return Ok(build_error_response(&format!("Failed to create container: {}", e))),
        };

        if let Err(e) = self
            .docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
        {
            return Ok(build_error_response(&format!("Failed to start container: {}", e)));
        }

        let wait_options = Some(WaitContainerOptions {
            condition: "not-running",
        });

        let mut wait_stream = self.docker.wait_container(&container.id, wait_options);

        while let Some(_) = wait_stream.next().await {}

        let logs = match self.docker.logs::<String>(
            &container.id,
            Some(bollard::container::LogsOptions {
                stdout: true,
                stderr: true,
                ..Default::default()
            }),
        ).try_collect::<Vec<_>>().await {
            Ok(logs) => logs.into_iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n"),
            Err(e) => format!("Failed to get logs: {}", e),
        };

        let _ = self.docker.remove_container(&container.id, None).await;

        Ok(build_success_response(serde_json::json!({ "output": logs })))
    }
}
