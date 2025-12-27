# Changelog

This document tracks changes made in this fork of [sigoden/aichat](https://github.com/sigoden/aichat).

## Fork Overview

This fork extends the original aichat with additional features focused on:
- Enhanced shell command execution with safety controls
- Improved output formatting options
- Dynamic model management
- Better Ollama integration

## Changes Since Fork

### Features Added

#### Shell Command Execution Enhancements

- **Yolo Mode (`-y`, `-yy`, `-yyy`)**: Added progressive yolo mode flags for executing commands without confirmation
  - `-y`: Safe mode - blocks root commands, warns on sudo usage
  - `-yy`: Allows root commands with warnings
  - `-yyy`: Full yolo mode - no safety checks
  - Includes improved dangerous command pattern detection to reduce false positives
  - Enhanced safety checks and distrobox role description

- **Distrobox Mode (`-d`)**: Added `--distrobox` flag to execute commands in distrobox/docker/podman containers
  - Includes safety checks for containerized execution
  - Improved pattern matching and root detection logic

#### Output Formatting

- **Output Format Conversion Flags**: Added flags to convert output to different formats
  - `--json`: Convert output to JSON format
  - `--yaml`: Convert output to YAML format
  - `--plain`: Output plain text without markdown rendering
  - Optimized regex performance and fixed output handling

#### Model Management

- **Models.dev API Integration**: Migrated from hardcoded `models.yaml` to dynamic loading from models.dev API
  - Automatic fallback to embedded `models.yaml` if API is unavailable
  - Configurable via environment variables
  - 1-hour cache TTL for API responses
  - Supports all providers available on models.dev

- **Model Refresh Command**: Added `--refresh-models` flag to refresh model lists for configured clients
  - Allows manual synchronization of model lists
  - Useful when new models are added to providers

#### Thinking Content Control

- **Hide Thinking Option**: Added `--hide-thinking` flag and `hide_thinking` config option
  - Strips `<think>` and `</think>` tags from model output
  - Works in both streaming and non-streaming modes
  - Configurable via CLI flag or config file

#### Ollama Integration (from blob42/aichat-ng)

- **Native Ollama Client**: Dedicated Ollama API client implementation
  - Uses native `/api/chat` and `/api/embed` endpoints
  - Better performance and reliability compared to OpenAI-compatible mode
  - Improved error handling

#### REPL Enhancements (from blob42/aichat-ng)

- **Path Autocompletion**: Enhanced REPL autocompletion for file paths
  - Tab completion for file and directory paths
  - Improved user experience when working with local files

### Bug Fixes

- Fixed array bounds and unsafe `unwrap()` calls
- Improved dangerous command patterns to reduce false positives
- Fixed pattern matching and root detection logic
- Enhanced safety checks for shell command execution
- Fixed output handling for format conversion flags

### Code Quality Improvements

- Migrated from hardcoded models.yaml to models.dev API
- Improved error handling and safety checks
- Optimized regex performance
- Better code organization and structure

## Upstream Compatibility

This fork maintains compatibility with the upstream [sigoden/aichat](https://github.com/sigoden/aichat) repository. All original features and functionality are preserved, with additional enhancements added on top.

## Contributing

Contributions are welcome! Please ensure that:
1. All tests pass (`cargo test --all`)
2. Code is formatted (`cargo fmt --all`)
3. Clippy checks pass (`cargo clippy --all --all-targets`)
4. New features are documented in this CHANGELOG

## Credits

- Original project: [sigoden/aichat](https://github.com/sigoden/aichat)
- Ollama and REPL enhancements: [blob42/aichat-ng](https://github.com/blob42/aichat-ng)

