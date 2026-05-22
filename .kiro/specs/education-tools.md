# Education Tools — Implementation Spec

## Overview

Five educational tools designed for an AI agent that teaches children. These tools generate visual, audio, and interactive learning materials by orchestrating existing media generation capabilities.

All tools live in a new crate: `adk-rust-mcp-education`

---

## 1. Whiteboard Generator (`whiteboard_generate`)

**Purpose:** Generate annotated diagrams, math solutions, and visual explanations as if drawn on a classroom whiteboard.

### Flow
```
Topic/Problem → Gemini Image Gen (whiteboard-style prompt) → Output PNG
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `content` | string | Yes | — | What to draw (math problem, diagram, concept map) |
| `style` | string | No | "whiteboard" | Style: whiteboard, blackboard, notebook, colorful |
| `annotations` | array | No | — | Extra labels/arrows to add `[{text, position}]` |
| `show_steps` | bool | No | false | Show step-by-step solution (for math) |
| `narration` | bool | No | false | Generate TTS explaining the board |
| `output_file` | string | No | — | Save path (.png or .mp4 if narrated) |

### Examples
```json
{"content": "Solve 3x + 7 = 22 step by step", "style": "whiteboard", "show_steps": true}
{"content": "Diagram of the water cycle with labels", "style": "colorful"}
{"content": "Food chain: sun → grass → rabbit → fox → decomposers", "style": "blackboard"}
```

### Output
- PNG image (whiteboard style with handwritten-look text and diagrams)
- If `narration: true`: MP4 video with static board + TTS explanation

### Implementation Notes
- Prompt engineering is key: include "hand-drawn style", "whiteboard marker", "step-by-step annotations"
- For math: prompt Gemini to show each step clearly numbered
- For diagrams: include "labeled arrows", "clear connections", "educational diagram"
- If narrated: generate TTS of the explanation, combine with static image via FFmpeg

---

## 2. Flashcard Generator (`flashcard_generate`)

**Purpose:** Generate a set of visual flashcards for studying a topic.

### Flow
```
Topic + Count → For each card:
  ├── Gemini (generate question + answer)
  └── Gemini Image Gen (visual for front)
→ Output: image set or PDF
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | Yes | — | Subject to create flashcards for |
| `count` | int | No | 5 | Number of cards (1-20) |
| `difficulty` | string | No | "easy" | easy, medium, hard |
| `age_group` | string | No | "8-10" | Target age range |
| `include_images` | bool | No | true | Generate images for card fronts |
| `output_dir` | string | No | — | Directory to save cards |
| `output_format` | string | No | "images" | "images" (individual PNGs) or "pdf" |

### Output Structure
```
flashcards/
├── card_01_front.png  (question + image)
├── card_01_back.png   (answer)
├── card_02_front.png
├── card_02_back.png
└── ...
```

### Examples
```json
{"topic": "Solar system planets", "count": 8, "age_group": "6-8", "difficulty": "easy"}
{"topic": "Times tables 1-12", "count": 12, "age_group": "7-9", "include_images": false}
{"topic": "Animal habitats", "count": 6, "difficulty": "medium"}
```

### Implementation Notes
- Step 1: Use Gemini text to generate Q&A pairs for the topic
- Step 2: For each card, generate an image (front) with the question/visual
- Step 3: Generate back image with the answer (text-based, clear and large)
- Card design: large text, bright colors, rounded corners, kid-friendly
- Prompt template: "A flashcard for children aged {age}. Front shows: {question}. Style: colorful, educational, clear, large text"

---

## 3. Story Generator (`story_generate`)

**Purpose:** Generate illustrated children's stories with narration — a complete picture book experience.

### Flow
```
Story Prompt → Gemini (generate story pages) → For each page:
  ├── Gemini Image Gen (illustration)
  └── Gemini TTS (narration)
→ FFmpeg: assemble into video → Output MP4
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Story idea or theme |
| `pages` | int | No | 5 | Number of pages/scenes (3-12) |
| `age_group` | string | No | "5-7" | Target age range |
| `style` | string | No | "watercolor" | Art style: watercolor, cartoon, pixel_art, storybook |
| `voice` | string | No | "Aoede" | Narrator voice |
| `moral` | string | No | — | Optional moral/lesson to include |
| `characters` | array | No | — | Character names/descriptions for consistency |
| `background_music` | string | No | "gentle lullaby" | Background music prompt |
| `output_file` | string | Yes | — | Output path (.mp4) |

### Examples
```json
{
  "prompt": "A brave little turtle who learns to swim",
  "pages": 6,
  "age_group": "4-6",
  "style": "watercolor",
  "moral": "Practice makes perfect",
  "background_music": "gentle acoustic guitar lullaby"
}
```

### Output
- MP4 video: each page shown for narration duration + 2s
- Each page: full-screen illustration + TTS narration
- Soft background music throughout
- Gentle crossfade transitions between pages

### Implementation Notes
- Step 1: Use Gemini text to generate the full story (page-by-page text)
- Step 2: Generate illustrations in parallel (maintain character consistency via detailed prompts)
- Step 3: Generate TTS for each page
- Step 4: Generate background music (Lyria Clip, 30s, loop)
- Step 5: Assemble with FFmpeg (same pattern as presentation_generate)
- Character consistency: include character descriptions in every image prompt
- Age-appropriate: adjust vocabulary and sentence length based on age_group

---

## 4. Quiz Generator (`quiz_generate`)

**Purpose:** Generate visual multiple-choice quizzes with illustrated options.

### Flow
```
Topic → Gemini (generate questions + options + answers) → For each question:
  └── Gemini Image Gen (question visual)
→ Output: image set with answer key
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | Yes | — | Quiz subject |
| `questions` | int | No | 5 | Number of questions (1-20) |
| `difficulty` | string | No | "easy" | easy, medium, hard |
| `age_group` | string | No | "8-10" | Target age range |
| `question_type` | string | No | "multiple_choice" | multiple_choice, true_false, fill_blank |
| `include_images` | bool | No | true | Generate images for questions |
| `narrated` | bool | No | false | Read questions aloud (TTS) |
| `output_dir` | string | No | — | Directory to save quiz |
| `output_file` | string | No | — | Single file output (.mp4 if narrated, .json for data) |

### Output Structure
```
quiz/
├── question_01.png    (question with options A-D)
├── question_02.png
├── ...
└── answer_key.json    (correct answers + explanations)
```

### Answer Key Format
```json
{
  "questions": [
    {
      "number": 1,
      "question": "Which planet is closest to the Sun?",
      "options": ["Venus", "Mercury", "Mars", "Earth"],
      "correct": "B",
      "explanation": "Mercury is the closest planet to the Sun."
    }
  ]
}
```

### Examples
```json
{"topic": "Dinosaurs", "questions": 5, "age_group": "6-8", "difficulty": "easy"}
{"topic": "Fractions", "questions": 10, "question_type": "fill_blank", "age_group": "9-11"}
{"topic": "World capitals", "questions": 8, "narrated": true, "output_file": "quiz.mp4"}
```

### Implementation Notes
- Step 1: Use Gemini to generate questions, options, correct answers, and explanations
- Step 2: Generate question images (show question text + options clearly)
- Step 3: If narrated, generate TTS for each question and assemble video
- Image style: clean, large text, colorful option boxes (A/B/C/D), kid-friendly
- Answer key always generated as JSON for programmatic use

---

## 5. Animated Explainer (`explainer_generate`)

**Purpose:** Generate step-by-step animated explanations of concepts — like a mini educational video.

### Flow
```
Topic → Gemini (break into steps) → For each step:
  ├── Gemini Image Gen (frame showing this step)
  └── Gemini TTS (explanation)
→ FFmpeg: assemble with transitions → Output MP4
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | Yes | — | Concept to explain |
| `steps` | int | No | auto | Number of steps (auto = let AI decide) |
| `age_group` | string | No | "8-10" | Target age range |
| `style` | string | No | "diagram" | diagram, cartoon, realistic, infographic |
| `voice` | string | No | "Kore" | Narrator voice |
| `pace` | string | No | "normal" | slow, normal, fast |
| `include_summary` | bool | No | true | Add summary slide at end |
| `background_music` | string | No | — | Optional background music |
| `output_file` | string | Yes | — | Output path (.mp4) |

### Examples
```json
{
  "topic": "How does photosynthesis work?",
  "age_group": "9-11",
  "style": "diagram",
  "voice": "Kore",
  "pace": "slow"
}
```
```json
{
  "topic": "How to add fractions with different denominators",
  "style": "whiteboard",
  "age_group": "10-12",
  "include_summary": true
}
```
```json
{
  "topic": "The life cycle of a butterfly",
  "style": "cartoon",
  "age_group": "5-7",
  "pace": "slow",
  "background_music": "cheerful xylophone"
}
```

### Output
- MP4 video (16:9)
- Each step: labeled diagram/illustration + narration
- Numbered steps visible on screen ("Step 1 of 5")
- Optional summary slide recapping all steps
- Transitions: crossfade between steps

### Implementation Notes
- Step 1: Use Gemini to break the topic into logical steps with explanations
- Step 2: Generate image for each step (include step number, labels, arrows)
- Step 3: Generate TTS for each step's explanation
- Step 4: Assemble video (same FFmpeg pattern as presentations)
- Pace control: "slow" adds 2s extra per step, "fast" removes padding
- Summary slide: text-only image listing all steps as bullet points
- Age adaptation: simpler language and more colorful visuals for younger kids

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  adk-rust-mcp-education                      │
│  whiteboard │ flashcard │ story │ quiz │ explainer           │
├─────────────────────────────────────────────────────────────┤
│              Gemini API (generateContent)                    │
├──────────┬──────────┬──────────┬──────────┬─────────────────┤
│  Image   │   Text   │   TTS    │  Music   │    FFmpeg       │
│  Gen     │   Gen    │  (voice) │ (Lyria)  │   (assembly)    │
└──────────┴──────────┴──────────┴──────────┴─────────────────┘
```

### Key Design Decisions

1. **Self-contained** — Calls Gemini API directly (like composer), no inter-crate deps
2. **Age-aware prompting** — All tools adapt language, complexity, and visuals to age_group
3. **Consistent characters** — Story generator maintains character descriptions across pages
4. **Structured data** — Quiz and flashcard tools output JSON alongside images for programmatic use
5. **Narration optional** — All tools can output static images or narrated video

---

## Crate Structure

```
adk-rust-mcp-education/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── server.rs          # MCP ServerHandler with 5 tools
│   ├── whiteboard.rs      # whiteboard_generate
│   ├── flashcard.rs       # flashcard_generate
│   ├── story.rs           # story_generate
│   ├── quiz.rs            # quiz_generate
│   └── explainer.rs       # explainer_generate
└── tests/
    └── integration_test.rs
```

---

## Priority Order

1. **Whiteboard** — Simplest (single image gen with smart prompting)
2. **Flashcard** — Text gen + image gen, structured output
3. **Explainer** — Step-by-step video (reuses presentation pattern)
4. **Story** — Multi-page illustrated video (similar to presentation)
5. **Quiz** — Most complex (structured data + images + optional video)

---

## Prompt Engineering Patterns

### Age-Appropriate Language
```
For age 4-6: "Use very simple words. Short sentences. Bright, friendly images."
For age 7-9: "Use clear language. Explain new words. Colorful educational style."
For age 10-12: "Use grade-appropriate vocabulary. Include details. Clean infographic style."
```

### Visual Consistency
```
"Style: {style}. For children aged {age_group}. 
 Educational, clear labels, bright colors, no scary elements.
 Large readable text. Simple backgrounds."
```

### Character Consistency (Stories)
```
"Character: {name} is a {description}. 
 Always draw {name} with {distinctive_features}.
 Art style: {style}, consistent with previous pages."
```
