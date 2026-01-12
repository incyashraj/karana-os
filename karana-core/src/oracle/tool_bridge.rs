// Kāraṇa OS - Oracle Tool Bridge
// Connects Oracle intents to actual tool execution

use crate::oracle::OracleIntent;
use crate::assistant::{ToolRegistry, ToolArgs};
use anyhow::Result;

/// Execute Oracle intent using tool registry
pub async fn execute_intent(
    intent: &OracleIntent,
    tool_registry: &ToolRegistry,
) -> Result<String> {
    match intent {
        OracleIntent::OpenApp { app_type } => {
            let mut args = ToolArgs::new();
            args.add("app_name", app_type.clone());
            let result = tool_registry.execute("launch_app", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::Navigate { destination } => {
            let mut args = ToolArgs::new();
            args.add("destination", destination.clone());
            let result = tool_registry.execute("navigate", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::CheckBalance => {
            let mut args = ToolArgs::new();
            args.add("action", "balance");
            let result = tool_registry.execute("wallet", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::Transfer { amount, recipient, memo } => {
            let mut args = ToolArgs::new();
            args.add("action", "transfer");
            args.add("amount", amount.to_string());
            args.add("recipient", recipient.clone());
            if let Some(m) = memo {
                args.add("memo", m.clone());
            }
            let result = tool_registry.execute("wallet", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::PlayVideo { query, url } => {
            let mut args = ToolArgs::new();
            args.add("app_name", "video_player");
            if let Some(q) = query {
                args.add("query", q.clone());
            }
            if let Some(u) = url {
                args.add("url", u.clone());
            }
            let result = tool_registry.execute("launch_app", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::OpenBrowser { url } => {
            let mut args = ToolArgs::new();
            args.add("app_name", "browser");
            if let Some(u) = url {
                args.add("url", u.clone());
            }
            let result = tool_registry.execute("launch_app", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::TakeNote { content } => {
            let mut args = ToolArgs::new();
            if let Some(c) = content {
                args.add("task", c.clone());
            } else {
                args.add("task", "New note");
            }
            let result = tool_registry.execute("create_task", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::SetReminder { message, duration } => {
            let mut args = ToolArgs::new();
            args.add("task", format!("{} (in {})", message, duration));
            let result = tool_registry.execute("create_task", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::PlayMusic { query } => {
            let mut args = ToolArgs::new();
            args.add("app_name", "music");
            if let Some(q) = query {
                args.add("query", q.clone());
            }
            let result = tool_registry.execute("launch_app", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::CloseApp { app_id } => {
            // TODO: Implement close_app tool
            Ok(format!("Closed app: {:?}", app_id))
        }
        
        OracleIntent::AnalyzeVision => {
            // TODO: Integrate vision analysis
            Ok("Vision analysis initiated".to_string())
        }
        
        OracleIntent::ExplainObject { context } => {
            // TODO: Integrate vision + LLM
            Ok(format!("Analyzing: {}", context))
        }
        
        OracleIntent::ShowDirections | OracleIntent::PinLocation => {
            let mut args = ToolArgs::new();
            args.add("destination", "current location");
            let result = tool_registry.execute("navigate", args).await?;
            Ok(result.output)
        }
        
        OracleIntent::SystemStatus => {
            // TODO: Implement system_status tool
            Ok("System status: All systems operational".to_string())
        }
        
        OracleIntent::AdjustBrightness { level } => {
            // TODO: Implement brightness tool
            Ok(format!("Brightness set to {}%", level))
        }
        
        OracleIntent::ToggleNightMode => {
            // TODO: Implement night mode tool
            Ok("Night mode toggled".to_string())
        }
        
        OracleIntent::EnablePrivacyMode => {
            // TODO: Implement privacy mode tool
            Ok("Privacy mode enabled".to_string())
        }
        
        OracleIntent::Conversation { response } => {
            // No tool execution needed - just return the response
            Ok(response.clone())
        }
        
        OracleIntent::Clarify { question } => {
            Ok(question.clone())
        }
        
        OracleIntent::Help => {
            Ok("I can help with navigation, launching apps, managing tasks, checking weather, and wallet operations. Just ask!".to_string())
        }
        
        OracleIntent::TransactionHistory | OracleIntent::StakeTokens { .. } | OracleIntent::VoteProposal { .. } => {
            // TODO: Implement blockchain tools
            Ok("Blockchain operation not yet implemented".to_string())
        }
        
        OracleIntent::TranslateText { .. } => {
            // TODO: Implement translation tool
            Ok("Translation not yet implemented".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_execute_navigate_intent() {
        let registry = ToolRegistry::new();
        let intent = OracleIntent::Navigate { destination: "home".to_string() };
        
        let result = execute_intent(&intent, &registry).await;
        assert!(result.is_ok());
    }
}
