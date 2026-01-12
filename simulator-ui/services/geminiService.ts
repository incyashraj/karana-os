import { AnalysisResult, OracleIntent } from "../types";

// Mock "Local" AI Service
const MODEL_NAME = "KARANA-NATIVE-V1";

// Simulated Knowledge Base for Vision
const MOCK_VISION_RESULTS: AnalysisResult[] = [
  {
    detectedObject: "Quantum Processor",
    category: "Hardware",
    description: "Advanced computing unit with holographic interface capabilities.",
    confidence: 98,
    relatedTags: ["tech", "compute", "future"]
  },
  {
    detectedObject: "Bio-Luminescent Plant",
    category: "Flora",
    description: "Genetically modified organism emitting natural light.",
    confidence: 92,
    relatedTags: ["nature", "biotech", "organic"]
  },
  {
    detectedObject: "Smart Coffee Cup",
    category: "Everyday Item",
    description: "Ceramic vessel with embedded temperature control sensors.",
    confidence: 89,
    relatedTags: ["iot", "beverage", "smart-home"]
  },
  {
    detectedObject: "Unknown Artifact",
    category: "Mystery",
    description: "Unidentified object with unusual energy signature.",
    confidence: 45,
    relatedTags: ["anomaly", "scan-required", "caution"]
  }
];

// 1. Vision Intelligence (Mocked Local Model)
export const analyzeImage = async (base64Image: string): Promise<AnalysisResult> => {
  console.log(`[${MODEL_NAME}] Processing visual data locally...`);
  
  // Simulate processing delay
  await new Promise(resolve => setTimeout(resolve, 1500));

  // Return a random result from our "knowledge base"
  const result = MOCK_VISION_RESULTS[Math.floor(Math.random() * MOCK_VISION_RESULTS.length)];
  
  return result;
};

// 2. Oracle Layer (Mocked Intent Processing)
export const processOracleIntent = async (userText: string, context?: string): Promise<OracleIntent> => {
  console.log(`[${MODEL_NAME}] Processing intent: "${userText}" Context: ${context}`);
  
  // Simulate processing delay
  await new Promise(resolve => setTimeout(resolve, 800));

  const lowerText = userText.toLowerCase();

  // Simple Keyword-based Intent Classification (Simulating NLU)
  
  // TRANSFER
  if (lowerText.includes("transfer") || lowerText.includes("send") || lowerText.includes("pay")) {
    const amountMatch = userText.match(/\d+/);
    const amount = amountMatch ? parseInt(amountMatch[0]) : 0;
    // Simple heuristic for recipient (word after "to")
    const recipientMatch = lowerText.match(/to\s+(\w+)/);
    const recipient = recipientMatch ? recipientMatch[1] : "Unknown";

    return {
      type: "TRANSFER",
      content: `Initiating secure transfer of ${amount} KARA to ${recipient}. Please confirm identity.`,
      data: { amount, recipient }
    };
  }

  // ANALYZE
  if (lowerText.includes("scan") || lowerText.includes("what is") || lowerText.includes("identify") || lowerText.includes("look at")) {
    return {
      type: "ANALYZE",
      content: "Activating visual cortex. Scanning environment...",
      data: {}
    };
  }

  // NAVIGATE
  if (lowerText.includes("navigate") || lowerText.includes("go to") || lowerText.includes("directions")) {
    const location = userText.replace(/navigate to|go to|directions to/i, "").trim();
    return {
      type: "NAVIGATE",
      content: `Calculating optimal path to ${location}. HUD navigation initialized.`,
      data: { location }
    };
  }

  // WALLET
  if (lowerText.includes("wallet") || lowerText.includes("balance") || lowerText.includes("money")) {
    return {
      type: "WALLET",
      content: "Accessing sovereign identity vault. Displaying assets.",
      data: {}
    };
  }

  // TIMER
  if (lowerText.includes("timer") || lowerText.includes("alarm")) {
    const duration = userText.replace(/set a timer for|timer/i, "").trim();
    return {
      type: "TIMER",
      content: `Timer set for ${duration}. I will notify you when it concludes.`,
      data: { duration }
    };
  }

  // DEFAULT / CONVERSATIONAL
  return {
    type: "SPEAK",
    content: `I am the Kāraṇa Local Oracle (v1.0). I processed your request: "${userText}" locally on-device. How can I assist you further?`,
    data: {}
  };
};
