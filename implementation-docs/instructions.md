# 250816

## Spider MCP client & repository

### MCP client

Spider is an MCP client.
It supports connecting to MCP servers, sending messages to LLMs, and doing tool loops in order to provide a full agentic MCP client.
It uses the https://github.com/hyperware-ai/hyperware-anthropic-sdk to message the Anthropic API.
It is designed to support other APIs as well (OpenAI and Google Gemini at a minimum).
Initial MVP with only include Anthropic API, but it must be designed with generality in mind.
In particular, a general message, conversation, tool, etc format must be used that can be easily converted to and from each of the API types.

Spider must support an agentic tool use loop where a request is made, the LLM decides to use a tool, the tool is used and result sent back to LLM, repeat arbitrarily many times before the LLM signals a final response.

Spider must support loading in API keys for each supported API (start with Anthropic API & Claude subscription only).
Spider must therefore serve a FE page to set and update LLM API keys.

Spider must support HTTP requests as well as `#[local]` p2p requests.
The HTTP requests must have a valid Spider API key associated with the request.
Spider must therefore also serve a FE page to create and update Spider API keys.

### Repository of requests and responses

Spider is a storehouse for requests and responses from past conversations.
Requests to Spider are decorated with metadata that allows requests and responses to be stored in a way that is easy to retrieve in the future.
Store conversations in the `conversations` drive in `jsonl` files named with the date and time like so:

```
spider:ware.hypr/
  conversations/
    20250816-141005.jsonl
    ...
```

Metadata fields:
```
start_time: String,
client: String,
from_stt: bool,
```

