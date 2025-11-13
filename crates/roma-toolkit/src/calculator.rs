use async_trait::async_trait;
use rig::tool::Tool;
use roma_core::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::base::{build_error_response, build_success_response, DynTool, Toolkit};

pub struct CalculatorToolkit;

impl CalculatorToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl Toolkit for CalculatorToolkit {
    fn name(&self) -> &str {
        "calculator"
    }

    fn tools(&self) -> Vec<DynTool> {
        vec![
            DynTool::new(AddTool),
            DynTool::new(SubtractTool),
            DynTool::new(MultiplyTool),
            DynTool::new(DivideTool),
        ]
    }
}

#[derive(Clone)]
struct AddTool;

#[derive(Deserialize, Serialize)]
struct MathArgs {
    x: f64,
    y: f64,
}

impl Tool for AddTool {
    const NAME: &'static str = "add";

    type Error = crate::base::ToolError;
    type Args = MathArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Add two numbers together".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "First number" },
                    "y": { "type": "number", "description": "Second number" }
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        let result = args.x + args.y;
        Ok(build_success_response(serde_json::json!({ "result": result })))
    }
}

#[derive(Clone)]
struct SubtractTool;

impl Tool for SubtractTool {
    const NAME: &'static str = "subtract";

    type Error = crate::base::ToolError;
    type Args = MathArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Subtract y from x".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "First number" },
                    "y": { "type": "number", "description": "Second number" }
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        let result = args.x - args.y;
        Ok(build_success_response(serde_json::json!({ "result": result })))
    }
}

#[derive(Clone)]
struct MultiplyTool;

impl Tool for MultiplyTool {
    const NAME: &'static str = "multiply";

    type Error = crate::base::ToolError;
    type Args = MathArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Multiply two numbers together".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "First number" },
                    "y": { "type": "number", "description": "Second number" }
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        let result = args.x * args.y;
        Ok(build_success_response(serde_json::json!({ "result": result })))
    }
}

#[derive(Clone)]
struct DivideTool;

impl Tool for DivideTool {
    const NAME: &'static str = "divide";

    type Error = crate::base::ToolError;
    type Args = MathArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Divide x by y".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "Numerator" },
                    "y": { "type": "number", "description": "Denominator (cannot be zero)" }
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        if args.y == 0.0 {
            return Ok(build_error_response("Division by zero"));
        }
        let result = args.x / args.y;
        Ok(build_success_response(serde_json::json!({ "result": result })))
    }
}
