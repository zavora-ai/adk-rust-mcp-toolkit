# adk-rust-mcp-education

MCP server for AI-powered educational content generation. Part of the ADK Rust MCP toolkit.

## Overview

Generate interactive learning materials for children — whiteboard diagrams, flashcards, illustrated stories, quizzes, and animated explainers. All tools adapt language and visuals to the target age group.

## Features

- **📝 Whiteboard** — Math solutions, diagrams, concept maps in hand-drawn style
- **🃏 Flashcards** — Visual Q&A card sets from any topic
- **📖 Stories** — Illustrated narrated children's stories as video
- **❓ Quizzes** — Multiple-choice with images and answer keys
- **🎬 Explainers** — Step-by-step animated concept explanations

## Installation

```bash
cargo install adk-rust-mcp-education
```

## Configuration

```bash
export GEMINI_API_KEY=your-api-key  # from https://aistudio.google.com/apikey
```

## Tools

### whiteboard_generate

Generate annotated diagrams, math solutions, and visual explanations.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `content` | string | Yes | — | What to draw (math problem, diagram, concept) |
| `style` | string | No | "whiteboard" | whiteboard, blackboard, notebook, colorful |
| `show_steps` | bool | No | false | Show step-by-step solution |
| `narration` | bool | No | false | Add TTS explanation (outputs .mp4) |
| `output_file` | string | No | — | Save path |

### flashcard_generate

Generate visual flashcard sets for studying.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | Yes | — | Subject for flashcards |
| `count` | int | No | 5 | Number of cards (1-20) |
| `difficulty` | string | No | "easy" | easy, medium, hard |
| `age_group` | string | No | "8-10" | Target age range |
| `include_images` | bool | No | true | Generate images for fronts |
| `output_dir` | string | No | — | Directory to save cards |

### story_generate

Generate illustrated children's stories with narration.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Story idea or theme |
| `pages` | int | No | 5 | Number of pages (3-12) |
| `age_group` | string | No | "5-7" | Target age range |
| `style` | string | No | "watercolor" | watercolor, cartoon, pixel_art, storybook |
| `voice` | string | No | "Aoede" | Narrator voice |
| `moral` | string | No | — | Optional lesson to include |
| `background_music` | string | No | "gentle lullaby" | Music prompt |
| `output_file` | string | Yes | — | Output path (.mp4) |

### quiz_generate

Generate visual multiple-choice quizzes with answer keys.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | Yes | — | Quiz subject |
| `questions` | int | No | 5 | Number of questions (1-20) |
| `difficulty` | string | No | "easy" | easy, medium, hard |
| `age_group` | string | No | "8-10" | Target age range |
| `question_type` | string | No | "multiple_choice" | multiple_choice, true_false, fill_blank |
| `include_images` | bool | No | true | Generate question images |
| `output_dir` | string | No | — | Directory to save quiz |

### explainer_generate

Generate step-by-step animated explanations of concepts.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | Yes | — | Concept to explain |
| `steps` | int | No | auto | Number of steps (0 = auto) |
| `age_group` | string | No | "8-10" | Target age range |
| `style` | string | No | "diagram" | diagram, cartoon, realistic, infographic |
| `voice` | string | No | "Kore" | Narrator voice |
| `pace` | string | No | "normal" | slow, normal, fast |
| `include_summary` | bool | No | true | Add summary slide |
| `background_music` | string | No | — | Optional music prompt |
| `output_file` | string | Yes | — | Output path (.mp4) |

## Example Outputs

### Whiteboard

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/whiteboard_math.png" width="500" alt="Math whiteboard"/>

### Flashcards

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/card_01_front.png" width="300" alt="Flashcard front"/>

### Quiz

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/question_01.png" width="400" alt="Quiz question"/>

### Story & Explainer

Generated as narrated MP4 videos with illustrations and background music.

## License

Apache-2.0
