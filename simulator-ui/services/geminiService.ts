import { GoogleGenAI, Type, Schema } from "@google/genai";
import { AnalysisResult, OracleIntent } from "../types";

const ai = new GoogleGenAI({ apiKey: process.env.API_KEY });

const MODEL_NAME = "gemini-2.5-flash";

// 1. Vision Intelligence (BLIP Equivalent)
export const analyzeImage = async (base64Image: string): Promise<AnalysisResult> => {
  try {
    const cleanBase64 = base64Image.replace(/^data:image\/(png|jpeg|jpg);base64,/, "");

    const response = await ai.models.generateContent({
      model: MODEL_NAME,
      contents: {
        parts: [
          {
            inlineData: {
              mimeType: "image/jpeg",
              data: cleanBase64,
            },
          },
          {
            text: "You are the Vision Layer of Kāraṇa OS. Analyze this image. Identify the main object. Return JSON: 'detectedObject' (short name), 'category', 'description' (1 short sentence), 'confidence' (0-100), 'relatedTags' (3 tags).",
          },
        ],
      },
      config: {
        responseMimeType: "application/json",
        responseSchema: {
          type: Type.OBJECT,
          properties: {
            detectedObject: { type: Type.STRING },
            category: { type: Type.STRING },
            description: { type: Type.STRING },
            confidence: { type: Type.NUMBER },
            relatedTags: {
              type: Type.ARRAY,
              items: { type: Type.STRING },
            },
          },
        },
      },
    });

    if (response.text) {
      return JSON.parse(response.text) as AnalysisResult;
    }
    throw new Error("No response text");
  } catch (error) {
    console.error("Analysis failed", error);
    throw error;
  }
};

// 2. Oracle Layer (Intent Processing)
export const processOracleIntent = async (userText: string, context?: string): Promise<OracleIntent> => {
  try {
    const systemPrompt = `
      You are the Oracle Layer of Kāraṇa OS, a sovereign operating system for smart glasses.
      Your job is to translate natural language into specific System Intents.
      
      Available Intents:
      - TRANSFER: sending tokens/money. (Extract amount and recipient)
      - ANALYZE: user asks "what is this?", "scan", "identify", "what am I looking at".
      - NAVIGATE: user wants directions. (Extract location)
      - TIMER: user wants to set a timer. (Extract duration)
      - WALLET: user wants to check balance, view transactions, or open wallet.
      - SPEAK: General conversation or questions about the OS/Blockchain/Identity.

      Context: ${context || "User is in idle mode."}
      
      Output JSON matching the schema.
    `;

    const schema: Schema = {
      type: Type.OBJECT,
      properties: {
        type: { 
          type: Type.STRING, 
          enum: ["SPEAK", "TRANSFER", "ANALYZE", "NAVIGATE", "TIMER", "WALLET"] 
        },
        content: { type: Type.STRING, description: "A short, HUD-friendly response text." },
        data: {
          type: Type.OBJECT,
          properties: {
            amount: { type: Type.NUMBER },
            recipient: { type: Type.STRING },
            location: { type: Type.STRING },
            duration: { type: Type.STRING },
          }
        }
      },
      required: ["type", "content"]
    };

    const response = await ai.models.generateContent({
      model: MODEL_NAME,
      contents: { parts: [{ text: userText }] },
      config: {
        systemInstruction: systemPrompt,
        responseMimeType: "application/json",
        responseSchema: schema
      }
    });

    if (response.text) {
      return JSON.parse(response.text) as OracleIntent;
    }
    return { type: 'SPEAK', content: "I didn't catch that." };

  } catch (error) {
    console.error("Oracle processing failed", error);
    return { type: 'SPEAK', content: "Oracle Layer offline." };
  }
};