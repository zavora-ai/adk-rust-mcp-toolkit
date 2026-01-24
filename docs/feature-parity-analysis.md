# Feature Parity Analysis: Rust MCP Servers vs Reference Project

## Summary

| Feature Category | Reference (Python) | Rust MCP | Status |
|-----------------|-------------------|----------|--------|
| Image Generation | ✅ | ✅ | ✅ Parity |
| Image Upscaling | ✅ | ✅ | ✅ Parity |
| Video Generation (T2V) | ✅ | ✅ | ✅ Parity |
| Video Generation (I2V) | ✅ | ✅ | ✅ Parity |
| Video Interpolation | ✅ | ✅ | ✅ Parity |
| Video Extension | ✅ | ✅ | ✅ Parity |
| Reference-to-Video (R2V) | ✅ | ❌ | 🔴 Missing |
| Music Generation (Lyria) | ✅ | ✅ | ✅ Parity |
| Speech (Chirp3-HD TTS) | ✅ | ✅ | ✅ Parity |
| Speech (Gemini TTS) | ✅ | ✅ | ✅ Parity |
| Virtual Try-On (VTO) | ✅ | ❌ | 🔴 Missing |
| Video Processing (FFmpeg) | ✅ | ✅ | ✅ Parity |
| Character Consistency | ✅ | ❌ | 🔴 Missing |

## Detailed Analysis

### ✅ Features with Parity

#### 1. Image Generation (Imagen)
- **Reference**: `models/image_models.py` - `generate_images()`
- **Rust**: `adk-rust-mcp-image` - `image_generate` tool
- **Features**: prompt, negative_prompt, aspect_ratio, number_of_images, GCS output
- **Status**: Full parity

#### 2. Image Upscaling (Imagen 4.0 Upscale)
- **Reference**: `models/upscale.py` - `upscale_image()`
- **Rust**: `adk-rust-mcp-image` - `image_upscale` tool
- **Features**: image input (base64/file/GCS), upscale_factor (x2/x4), output to file/GCS/base64
- **Status**: Full parity

#### 3. Video Generation - Text-to-Video
- **Reference**: `models/veo.py` - `generate_video()` with text prompt
- **Rust**: `adk-rust-mcp-video` - `video_generate` tool
- **Features**: prompt, aspect_ratio, duration, generate_audio (Veo 3), GCS output
- **Status**: Full parity

#### 4. Video Generation - Image-to-Video
- **Reference**: `models/veo.py` - `generate_video()` with `reference_image_gcs`
- **Rust**: `adk-rust-mcp-video` - `video_from_image` tool
- **Features**: image input, prompt, aspect_ratio, duration, GCS output
- **Status**: Full parity

#### 5. Video Interpolation (First + Last Frame)
- **Reference**: `models/veo.py` - `generate_video()` with `reference_image_gcs` + `last_reference_image_gcs`
- **Rust**: `adk-rust-mcp-video` - `video_from_image` tool with `last_frame_image` parameter
- **Features**: first frame image, last frame image, prompt, interpolation between frames
- **Status**: Full parity

#### 6. Video Extension
- **Reference**: `models/veo.py` - `generate_video()` with `video_input_gcs`
- **Rust**: `adk-rust-mcp-video` - `video_extend` tool
- **Features**: video input (GCS URI), prompt, duration, GCS output
- **Status**: Full parity

#### 7. Music Generation (Lyria)
- **Reference**: `models/lyria.py` - `generate_music_with_lyria()`
- **Rust**: `adk-rust-mcp-music` - `music_generate` tool
- **Features**: prompt, sample_count, GCS output
- **Status**: Full parity

#### 8. Speech Synthesis (Chirp3-HD)
- **Reference**: `models/chirp_3hd.py` - `synthesize_chirp_speech()`
- **Rust**: `adk-rust-mcp-speech` - `speech_synthesize` tool
- **Features**: text, voice, language_code, speaking_rate, custom pronunciations
- **Status**: Full parity

#### 9. Speech Synthesis (Gemini TTS)
- **Reference**: `models/gemini_tts.py` - `synthesize_speech()`
- **Rust**: `adk-rust-mcp-multimodal` - `multimodal_speech_synthesize` tool
- **Features**: text, voice, style/prompt
- **Status**: Full parity

#### 10. FFmpeg Video Processing
- **Reference**: `models/video_processing.py` - various functions
- **Rust**: `adk-rust-mcp-avtool` - 8 FFmpeg tools
- **Features**: concatenate, crossfade, combine A/V, overlay, volume, GIF conversion
- **Status**: Full parity (Rust has more granular tools)

### 🔴 Missing Features

#### 1. Reference-to-Video (R2V) - Style & Asset References
- **Reference**: `models/veo.py` lines 127-145
```python
if request.r2v_style_image:
    reference_images_list.append(types.VideoGenerationReferenceImage(
        image=types.Image(gcs_uri=...),
        reference_type="style",
    ))
if request.r2v_references:
    for ref in request.r2v_references:
        reference_images_list.append(types.VideoGenerationReferenceImage(
            reference_type="asset",
        ))
```
- **Gap**: Rust video server doesn't support style/asset reference images

#### 2. Virtual Try-On (VTO)
- **Reference**: `models/vto.py` - `generate_vto_image()`
```python
response = client.models.recontext_image(
    model=model_id,
    source=RecontextImageSource(
        person_image=Image(gcs_uri=person_gcs_uri),
        product_images=[ProductImage(product_image=Image(gcs_uri=product_gcs_uri))],
    ),
)
```
- **Gap**: No VTO/recontext tool in Rust servers

#### 3. Character Consistency
- **Reference**: `models/character_consistency.py`
- **Gap**: No character consistency features in Rust servers

## Priority Recommendations

### Medium Priority (Advanced Features)
1. **R2V Style/Asset References** - Add reference image support to video server

### Lower Priority (Specialized Features)
2. **Virtual Try-On** - New server or tool for VTO/recontext
3. **Character Consistency** - Specialized feature for consistent characters

## Implementation Notes

### R2V Style/Asset References
Add to `VideoT2vParams` and `VideoI2vParams`:
```rust
pub style_reference_image: Option<String>,  // GCS URI for style reference
pub asset_references: Option<Vec<AssetReference>>,  // Asset reference images

pub struct AssetReference {
    pub image: String,  // GCS URI
    pub reference_type: String,  // "asset"
}
```
