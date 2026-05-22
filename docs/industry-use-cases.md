# Industry Use Cases

How to use the ADK Rust MCP Toolkit across 10 key industries. Each section shows which tools to use and example prompts.

---

## 1. Education & E-Learning

**Tools:** `whiteboard_generate`, `flashcard_generate`, `story_generate`, `quiz_generate`, `explainer_generate`, `presentation_generate`, `podcast_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Math tutoring | `whiteboard_generate` | "Solve step by step: find the area of a triangle with base 8cm and height 5cm" |
| Language flashcards | `flashcard_generate` | topic: "Spanish vocabulary for food and drinks", age_group: "10-12" |
| Science explainers | `explainer_generate` | "How do volcanoes erupt?" style: "diagram", age_group: "9-11" |
| History stories | `story_generate` | "The journey of a letter in ancient Rome", style: "storybook" |
| Assessment | `quiz_generate` | topic: "Photosynthesis", difficulty: "medium", questions: 10 |
| Lecture recordings | `presentation_generate` | slides about cell biology with narration |
| Language podcasts | `podcast_generate` | Conversational Spanish practice between teacher and student |

---

## 2. Healthcare & Medical

**Tools:** `explainer_generate`, `presentation_generate`, `whiteboard_generate`, `podcast_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Patient education | `explainer_generate` | "How does insulin work in the body?", style: "diagram", pace: "slow" |
| Procedure walkthroughs | `presentation_generate` | Slides explaining pre-surgery preparation steps |
| Anatomy diagrams | `whiteboard_generate` | "Diagram of the human heart with labeled chambers and blood flow" |
| Staff training | `explainer_generate` | "Proper hand hygiene protocol - 7 steps", style: "infographic" |
| Health podcasts | `podcast_generate` | Doctor-patient dialogue about managing diabetes |
| Drug interaction visuals | `whiteboard_generate` | "How aspirin works: mechanism of action diagram" |
| Wellness content | `short_generate` | "Calming breathing exercise demonstration, 4-7-8 technique" |

---

## 3. Real Estate & Property

**Tools:** `video_generate`, `short_generate`, `image_generate`, `presentation_generate`, `podcast_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Property flythroughs | `video_generate` | "Cinematic walkthrough of a modern luxury apartment with floor-to-ceiling windows, ocean view" |
| Listing shorts | `short_generate` | "Tour of a cozy 2-bedroom cottage with garden, warm lighting", caption: "Just Listed! 🏡" |
| Neighborhood visuals | `image_generate` | "Aerial view of a suburban neighborhood with parks, schools, and shopping nearby" |
| Market reports | `presentation_generate` | Slides: Q4 housing market trends, median prices, inventory levels |
| Agent podcasts | `podcast_generate` | Two agents discussing "Top 5 tips for first-time homebuyers" |
| Staging concepts | `image_generate` | "Modern minimalist living room staging, neutral tones, natural light" |

---

## 4. Marketing & Advertising

**Tools:** `short_generate`, `meme_generate`, `gif_generate`, `image_generate`, `music_generate`, `podcast_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Social media ads | `short_generate` | "Product reveal of a sleek wireless headphone, dramatic lighting, 360 spin" |
| Brand memes | `meme_generate` | prompt: "person choosing between two buttons", top: "Sleep", bottom: "One more episode" |
| Email GIFs | `gif_generate` | "Animated sale banner with confetti and 50% OFF text bouncing" |
| Product imagery | `image_generate` | "Flat lay of skincare products on marble surface, soft natural lighting" |
| Ad jingles | `music_generate` | "Upbeat 30-second jingle for a coffee brand, cheerful acoustic guitar" |
| Brand podcasts | `podcast_generate` | Host interviewing a founder about their startup journey |
| Campaign videos | `presentation_generate` | Product launch deck with narration and background music |

---

## 5. IT & Software

**Tools:** `whiteboard_generate`, `explainer_generate`, `presentation_generate`, `meme_generate`, `podcast_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Architecture diagrams | `whiteboard_generate` | "Microservices architecture: API gateway → auth service → user service → database" |
| Onboarding videos | `explainer_generate` | "How our CI/CD pipeline works: commit → build → test → deploy" |
| Sprint demos | `presentation_generate` | Slides showing new features with screenshots and narration |
| Dev culture memes | `meme_generate` | prompt: "developer surrounded by monitors", top: "IT WORKS ON MY MACHINE" |
| Tech podcasts | `podcast_generate` | Two engineers discussing "Kubernetes vs serverless for startups" |
| Incident postmortems | `whiteboard_generate` | "Timeline: 14:00 alert fired → 14:15 identified → 14:30 fix deployed" |
| API documentation | `explainer_generate` | "How OAuth 2.0 authorization code flow works", style: "diagram" |

---

## 6. Logistics & Supply Chain

**Tools:** `whiteboard_generate`, `explainer_generate`, `presentation_generate`, `quiz_generate`, `short_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Process flows | `whiteboard_generate` | "Order fulfillment flow: order received → pick → pack → ship → deliver" |
| Driver training | `explainer_generate` | "Safe loading procedures for a delivery truck - 5 steps" |
| Safety quizzes | `quiz_generate` | topic: "Warehouse safety protocols", difficulty: "medium" |
| Route optimization visuals | `whiteboard_generate` | "Hub and spoke distribution model with 3 regional warehouses" |
| Compliance training | `presentation_generate` | Slides on hazardous materials handling procedures |
| Social proof | `short_generate` | "Timelapse of a busy warehouse with packages moving on conveyors" |
| KPI dashboards | `whiteboard_generate` | "Supply chain metrics: lead time, fill rate, inventory turnover" |

---

## 7. Finance & Banking

**Tools:** `presentation_generate`, `explainer_generate`, `whiteboard_generate`, `podcast_generate`, `short_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Investor decks | `presentation_generate` | Slides: company growth, revenue projections, market opportunity |
| Financial literacy | `explainer_generate` | "How compound interest works", age_group: "16-18", style: "infographic" |
| Process diagrams | `whiteboard_generate` | "Loan approval workflow: application → credit check → underwriting → approval" |
| Market commentary | `podcast_generate` | Two analysts discussing weekly market movements |
| Product explainers | `explainer_generate` | "How a mortgage works: principal, interest, amortization" |
| Social content | `short_generate` | "Animated infographic showing savings growth over 10 years" |
| Compliance training | `quiz_generate` | topic: "Anti-money laundering regulations", difficulty: "hard" |

---

## 8. Retail & E-Commerce

**Tools:** `image_generate`, `short_generate`, `gif_generate`, `meme_generate`, `music_generate`, `presentation_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Product photos | `image_generate` | "White sneakers on a clean white background, studio lighting, e-commerce style" |
| Unboxing videos | `short_generate` | "Hands opening a premium gift box revealing a gold watch, satisfying" |
| Sale animations | `gif_generate` | "Animated countdown timer 3-2-1 with fireworks, BLACK FRIDAY text" |
| Social engagement | `meme_generate` | prompt: "person with shopping bags", top: "PAYDAY", bottom: "GONE IN 60 SECONDS" |
| Store ambiance | `music_generate` | "Relaxing lo-fi background music for a boutique clothing store" |
| Seasonal campaigns | `presentation_generate` | Holiday gift guide with product images and descriptions |
| Staff training | `explainer_generate` | "How to process returns and exchanges - step by step" |

---

## 9. Media & Entertainment

**Tools:** `video_generate`, `music_generate`, `music_realtime_start`, `gif_generate`, `short_generate`, `podcast_generate`, `image_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Concept trailers | `video_generate` | "Epic cinematic shot of a spaceship emerging from clouds, dramatic lighting" |
| Background scores | `music_generate` | "Tense orchestral score for a thriller movie trailer, building to climax" |
| Live DJ sets | `music_realtime_start` | prompts: [{text: "deep house, sunset vibes", weight: 1}], config: {bpm: 122} |
| Reaction GIFs | `gif_generate` | "Person doing an exaggerated surprised face, cartoon style" |
| Teasers/trailers | `short_generate` | "Mysterious figure walking through fog, cinematic color grading" |
| Interview podcasts | `podcast_generate` | Host interviewing a musician about their creative process |
| Album artwork | `image_generate` | "Abstract album cover, neon colors, geometric shapes, synthwave aesthetic" |

---

## 10. Hospitality & Travel

**Tools:** `video_generate`, `short_generate`, `image_generate`, `presentation_generate`, `music_generate`, `podcast_generate`

| Use Case | Tool | Example Prompt |
|----------|------|----------------|
| Destination videos | `video_generate` | "Aerial drone shot of a tropical beach resort at golden hour, crystal clear water" |
| Social reels | `short_generate` | "Cocktail being prepared at a rooftop bar with city skyline sunset", caption: "Friday vibes 🍸" |
| Menu visuals | `image_generate` | "Gourmet pasta dish on a rustic wooden table, warm restaurant lighting, food photography" |
| Travel guides | `presentation_generate` | Slides: "Top 10 things to do in Bali" with images and narration |
| Lobby music | `music_generate` | "Smooth jazz for a luxury hotel lobby, piano and soft saxophone, ambient" |
| Travel podcasts | `podcast_generate` | Two travelers sharing "Hidden gems of Southeast Asia" |
| Virtual tours | `video_generate` | "Walking tour through a boutique hotel, showing lobby, pool, and ocean-view suite" |

---

## Getting Started

All use cases above work with a single environment variable:

```bash
export GEMINI_API_KEY=your-key  # from https://aistudio.google.com/apikey
```

Then run the appropriate server:

```bash
# For education use cases
adk-rust-mcp-education --transport stdio

# For marketing/social media
adk-rust-mcp-composer --transport stdio

# For video content
adk-rust-mcp-video --transport stdio

# For all capabilities at once, configure multiple servers in your MCP client
```

See the [main README](../README.md) for full multi-server configuration.
