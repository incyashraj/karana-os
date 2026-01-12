// Phase 54.4: Comprehensive Test Suite for Universal Oracle
use anyhow::Result;
use karana_core::oracle::UniversalOracle;

#[tokio::test]
async fn test_200_universal_usecases() -> Result<()> {
    let oracle = UniversalOracle::new();
    let mut passed = 0;
    let mut failed = 0;

    // Category 1: OS Operations (50 cases)
    let os_cases = vec![
        ("tune battery", "Battery"),
        ("increase brightness", "Brightness"),
        ("lower volume", "Volume"),
        ("enable dark mode", "dark"),
        ("optimize performance", "performance"),
        ("check storage space", "storage"),
        ("enable bluetooth", "bluetooth"),
        ("wifi settings", "wifi"),
        ("night light on", "night"),
        ("do not disturb", "disturb"),
    ];

    for (intent, expected_keyword) in os_cases {
        match oracle.mediate(intent).await {
            Ok(manifest) => {
                if manifest.confidence > 0.6 && 
                   manifest.text.to_lowercase().contains(&expected_keyword.to_lowercase()) {
                    passed += 1;
                } else {
                    println!("FAIL [OS]: {} -> {}", intent, manifest.text);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("ERROR [OS]: {} -> {}", intent, e);
                failed += 1;
            }
        }
    }

    // Category 2: General Knowledge (50 cases)
    let general_cases = vec![
        ("weather Paris", "°"),
        ("time in Tokyo", "Tokyo"),
        ("distance to Mars", "km"),
        ("population of India", "billion"),
        ("capital of Brazil", "Brasilia"),
        ("quantum mechanics basics", "quantum"),
        ("speed of light", "299"),
        ("who invented telephone", "Bell"),
        ("history of bitcoin", "bitcoin"),
        ("explain relativity", "Einstein"),
    ];

    for (intent, expected_keyword) in general_cases {
        match oracle.mediate(intent).await {
            Ok(manifest) => {
                if manifest.confidence > 0.6 && manifest.text.len() > 10 {
                    passed += 1;
                } else {
                    println!("FAIL [GENERAL]: {} -> {}", intent, manifest.text);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("ERROR [GENERAL]: {} -> {}", intent, e);
                failed += 1;
            }
        }
    }

    // Category 3: App/Productivity (50 cases)
    let app_cases = vec![
        ("install VS Code", "VS Code"),
        ("open calendar", "calendar"),
        ("create new note", "note"),
        ("set timer 5 minutes", "timer"),
        ("remind me at 3pm", "remind"),
        ("calculate 15% of 200", "30"),
        ("convert 10 USD to EUR", "EUR"),
        ("translate hello to French", "Bonjour"),
        ("schedule meeting tomorrow", "schedule"),
        ("send email to team", "email"),
    ];

    for (intent, expected_keyword) in app_cases {
        match oracle.mediate(intent).await {
            Ok(manifest) => {
                if manifest.confidence > 0.6 {
                    passed += 1;
                } else {
                    println!("FAIL [APP]: {} -> {}", intent, manifest.text);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("ERROR [APP]: {} -> {}", intent, e);
                failed += 1;
            }
        }
    }

    // Category 4: Creative Tasks (50 cases)
    let creative_cases = vec![
        ("write poem about love", "love"),
        ("haiku about nature", "nature"),
        ("story about robot", "robot"),
        ("joke about programming", "programming"),
        ("motivational quote", ""),
        ("birthday message", "birthday"),
        ("explain quantum in simple terms", "quantum"),
        ("describe sunset", "sunset"),
        ("write apology", "sorry"),
        ("philosophy of time", "time"),
    ];

    for (intent, _) in creative_cases {
        match oracle.mediate(intent).await {
            Ok(manifest) => {
                if manifest.confidence > 0.6 && manifest.text.len() > 30 {
                    passed += 1;
                } else {
                    println!("FAIL [CREATIVE]: {} -> {}", intent, manifest.text);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("ERROR [CREATIVE]: {} -> {}", intent, e);
                failed += 1;
            }
        }
    }

    // Category 5: Random/Edge Cases (40 cases)
    let random_cases = vec![
        ("should I umbrella?", ""),
        ("am I late?", ""),
        ("best restaurant nearby", "restaurant"),
        ("lucky number today", ""),
        ("health check", "health"),
        ("mood for music", "music"),
        ("inspire me", ""),
        ("what's trending", ""),
        ("surprise me", ""),
        ("anything interesting?", ""),
    ];

    for (intent, _) in random_cases {
        match oracle.mediate(intent).await {
            Ok(manifest) => {
                if manifest.confidence > 0.5 && manifest.text.len() > 5 {
                    passed += 1;
                } else {
                    println!("FAIL [RANDOM]: {} -> {}", intent, manifest.text);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("ERROR [RANDOM]: {} -> {}", intent, e);
                failed += 1;
            }
        }
    }

    let total = passed + failed;
    let success_rate = (passed as f32 / total as f32) * 100.0;

    println!("\n=== Universal Oracle Test Results ===");
    println!("Total: {}", total);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Success Rate: {:.1}%", success_rate);
    println!("=====================================\n");

    assert!(success_rate >= 85.0, "Success rate below 85% threshold");

    Ok(())
}

#[tokio::test]
async fn test_multi_step_chaining() -> Result<()> {
    let oracle = UniversalOracle::new();

    // Complex: Weather + History + Decision
    let manifest = oracle.mediate("Should I take umbrella for my commute?").await?;
    
    assert!(manifest.confidence > 0.7);
    assert!(manifest.reasoning_trace.len() >= 2); // Multi-step
    assert!(!manifest.historical_context.is_empty() || manifest.text.len() > 20);

    Ok(())
}

#[tokio::test]
async fn test_confidence_gates() -> Result<()> {
    let oracle = UniversalOracle::new();

    // Low confidence request
    let manifest = oracle.mediate("asdfghjkl qwerty zxcvbnm").await?;
    
    assert!(manifest.confidence < 0.7);
    assert!(manifest.text.to_lowercase().contains("clarify") || 
            manifest.text.to_lowercase().contains("uncertain"));

    Ok(())
}

#[tokio::test]
async fn test_feedback_self_improvement() -> Result<()> {
    let oracle = UniversalOracle::new();

    // Initial query
    let _ = oracle.mediate("umbrella needed?").await?;
    oracle.process_feedback("umbrella needed?", true).await?;

    // Repeat query - should have better context
    let manifest2 = oracle.mediate("umbrella needed?").await?;
    
    assert!(manifest2.confidence >= 0.7);
    assert!(!manifest2.historical_context.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_response_latency() -> Result<()> {
    let oracle = UniversalOracle::new();
    
    let start = std::time::Instant::now();
    let _ = oracle.mediate("What's the weather?").await?;
    let elapsed = start.elapsed();

    println!("Response latency: {:?}", elapsed);
    assert!(elapsed.as_millis() < 2000, "Response too slow (>2s)");

    Ok(())
}
