#!/usr/bin/env node
/**
 * Mock Codex CLI for testing CoSurf integration
 * 
 * This simulates the Codex CLI behavior without requiring the actual binary.
 * Usage: node mock_codex_cli.js --json
 */

const readline = require('readline');

// Simulate Codex JSON response format
function createCodexResponse(text, eventType = 'agent.message.content.delta') {
  return JSON.stringify({
    type: eventType,
    content: text,
    timestamp: new Date().toISOString(),
  });
}

async function main() {
  const args = process.argv.slice(2);
  const isJsonMode = args.includes('--json');
  
  if (!isJsonMode) {
    console.error('Error: --json mode is required');
    process.exit(1);
  }
  
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  
  // Send initial greeting
  setTimeout(() => {
    process.stdout.write(createCodexResponse('👋 Hello! I am Codex (mock mode). How can I help you today?\n'));
  }, 500);
  
  // Listen for user messages
  rl.on('line', async (line) => {
    try {
      const message = JSON.parse(line);
      
      if (message.type === 'user_message') {
        const userText = message.content;
        
        // Simulate thinking delay
        await sleep(1000);
        
        // Send streaming response
        const responses = generateMockResponses(userText);
        
        for (const response of responses) {
          process.stdout.write(createCodexResponse(response));
          await sleep(200 + Math.random() * 300); // Random delay for realism
        }
        
        // Send completion signal
        process.stdout.write(JSON.stringify({
          type: 'agent.turn.completed',
          timestamp: new Date().toISOString(),
        }) + '\n');
      }
    } catch (error) {
      console.error('Error parsing input:', error.message);
    }
  });
}

function generateMockResponses(userText) {
  // Simple mock responses based on user input
  const lowerText = userText.toLowerCase();
  
  if (lowerText.includes('hello') || lowerText.includes('hi')) {
    return [
      'Hello! ',
      'How can I assist you today? ',
      'I can help with coding tasks, answering questions, or exploring codebases.',
    ];
  } else if (lowerText.includes('code') || lowerText.includes('programming')) {
    return [
      'I\'d be happy to help with programming! ',
      'Could you provide more details about what you\'re working on? ',
      'For example, are you looking to debug, refactor, or write new code?',
    ];
  } else if (lowerText.includes('help')) {
    return [
      'Sure! Here\'s how I can help:\n\n',
      '1. **Code Analysis** - Review and explain code\n',
      '2. **Debugging** - Find and fix issues\n',
      '3. **Refactoring** - Improve code quality\n',
      '4. **Documentation** - Write clear docs\n\n',
      'What would you like to work on?',
    ];
  } else {
    return [
      `You said: "${userText}"\n\n`,
      'That\'s interesting! ',
      'Could you elaborate on what you\'d like me to do with this information? ',
      'I can analyze, summarize, or help implement solutions.',
    ];
  }
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

main().catch(console.error);
