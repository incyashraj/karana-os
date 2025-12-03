#!/usr/bin/env python3
"""Test WebSocket streaming for Oracle events"""
import asyncio
import websockets
import json
import aiohttp

async def test_oracle_ws():
    print("???? Connecting to WebSocket...")
    
    # Connect to WebSocket
    async with websockets.connect("ws://localhost:8080/ws") as ws:
        # Subscribe to oracle channel
        subscribe_msg = json.dumps({"type": "subscribe", "channel": "oracle"})
        await ws.send(subscribe_msg)
        print("???? Subscribed to oracle channel")
        
        # Start listening for messages in background
        async def listen():
            while True:
                try:
                    msg = await asyncio.wait_for(ws.recv(), timeout=10.0)
                    data = json.loads(msg)
                    print(f"???? WS Event: {json.dumps(data, indent=2)}")
                except asyncio.TimeoutError:
                    print("?????? No more messages (timeout)")
                    break
                except Exception as e:
                    print(f"??? Error: {e}")
                    break
        
        # Start listener task
        listener = asyncio.create_task(listen())
        
        # Wait a moment for subscription to process
        await asyncio.sleep(0.5)
        
        # Send intent via HTTP
        print("\n??????? Sending intent: 'check my balance'...")
        async with aiohttp.ClientSession() as http:
            async with http.post(
                "http://localhost:8080/api/ai/oracle",
                json={"text": "check my balance"},
                headers={"Content-Type": "application/json"}
            ) as resp:
                result = await resp.json()
                print(f"???? HTTP Response: {json.dumps(result, indent=2)}")
        
        # Wait for WebSocket events
        await asyncio.sleep(2)
        
        # Send another intent requiring confirmation
        print("\n??????? Sending intent: 'send 50 KARA to bob'...")
        async with aiohttp.ClientSession() as http:
            async with http.post(
                "http://localhost:8080/api/ai/oracle",
                json={"text": "send 50 KARA to bob"},
                headers={"Content-Type": "application/json"}
            ) as resp:
                result = await resp.json()
                print(f"???? HTTP Response: {json.dumps(result, indent=2)}")
        
        # Wait for more WebSocket events
        await asyncio.sleep(3)
        
        listener.cancel()
        print("\n??? Test complete!")

if __name__ == "__main__":
    asyncio.run(test_oracle_ws())
