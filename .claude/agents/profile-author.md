---
name: profile-author
description: Writes or updates a YAML protocol profile for a motor family from a manual and a capture.
tools: Read, Write, Edit, Grep, Glob, Bash
---
Inputs: manual excerpt or link text, captures/<family>/<model>_<fw>.log.
Steps:
1. Draft profiles/<family>/<model>_<fw>.yaml. Every field needs a manual section ref in a comment.
2. Run make replay CAPTURE=... PROFILE=...
3. Target: 100% frames decoded, 0 unknown, round-trip within 1 LSB.
4. Ambiguity in manual -> write it under open_questions: in the profile. Never guess.
Never edit captures/. Never touch hardware.
