# spider

Spider is the central orchestrator for the web of LLM-powered applications on Hyperware

<img width="753" height="301" alt="image" src="https://github.com/user-attachments/assets/45c757bd-e4c8-417d-a195-312c4e50fdbb" />

## Status

[Read the roadmap here](https://gist.github.com/nick1udwig/117f9fc5bfd134f987183dd7c67343b4)

[Read a non-technical discussion of the vision here](https://gist.github.com/nick1udwig/147827a2d7d4f432ed186f6b2085a939)

## Building

Depends on https://github.com/hyperware-ai/anthropic-api-key-manager

First build and put that app on a fakenode; then build spider:

```
# In a terminal, start a fakenode:
kit f

# In another terminal, build and install the API key manager:
kit b --hyperapp && kit s

# Then build Spider:
cd ~/git/spider
kit b --hyperapp
```
