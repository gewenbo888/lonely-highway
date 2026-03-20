# Lonely Highway — Game Design Spec

## Overview

**Lonely Highway** is a realistic driving simulator set in a faithful recreation of Shenzhen, China. The player is a new arrival to the city, exploring its districts, roads, and atmosphere through freeform driving. Built in Unity (C#), medium indie scope.

**Core pillars:**
- Grounded realism — real traffic rules, real road feel, real city
- Faithful Shenzhen — OSM-based city generation, not a fictional stand-in
- Freeform exploration — no forced objectives, quiet milestones reward curiosity
- Atmospheric immersion — full day-night cycle, weather, reactive soundscape

---

## 1. System Architecture

Six major systems:

1. **OSM Pipeline** (offline tool) — Fetches Shenzhen OSM data, generates Unity-ready assets
2. **World Streaming** — Loads/unloads tile chunks around the player
3. **Vehicle Physics** — Custom raycast vehicle with Pacejka tire model
4. **Traffic AI** — Dense reactive traffic with lane-graph pathfinding
5. **Environment** — Day-night cycle, weather, lighting, ambient audio
6. **Progression** — Stats tracking, milestones, save system

### System relationships

- OSM Pipeline outputs tiles consumed by World Streaming
- World Streaming provides road surface data to Vehicle Physics and lane graphs to Traffic AI
- Traffic AI is aware of the player vehicle (React layer)
- Environment modifies World Streaming visuals (wet roads, lighting) and Traffic AI density (rush hour vs night)
- Vehicle Physics feeds driving stats into Progression

---

## 2. OSM Pipeline

An offline tool that converts real Shenzhen geographic data into Unity assets.

### Data sources

- **OpenStreetMap via Overpass API** — roads (lane count, speed limits, surface type), buildings (footprints + height tags), land use zones, traffic signals, crosswalks
- **SRTM/ASTER elevation data** — terrain height for hills and overpasses

### Processing stages

1. **Fetch** — Download OSM data for a bounding box covering Shenzhen (or specific districts)
2. **Parse** — Extract road centerlines, building polygons, signal positions, lane metadata
3. **Road mesh generation** — Extrude centerlines into road surfaces with lane widths, curbs, medians, elevated sections. Tag each segment with surface type (asphalt, concrete) for physics
4. **Building shell generation** — Extrude footprints to height. Classify by OSM tags (residential, commercial, industrial) for facade variation
5. **Traffic graph generation** — Build a directed lane graph with connections, signal phases, speed limits. This is what Traffic AI pathfinds on
6. **Tile chunking** — Divide the city into a grid (256m x 256m tiles). Each tile becomes a Unity scene with road meshes, building meshes, and traffic graph data as ScriptableObjects
7. **Export** — glTF meshes + ScriptableObjects + tile metadata JSON

### Language

Rust (consistent with the arnis reference project). Could also be Python with osmium + trimesh.

### Output per tile

- `tile_x_y.glb` — road + building meshes
- `tile_x_y_traffic.asset` — lane graph ScriptableObject
- `tile_x_y_meta.json` — bounds, LOD info, signal positions

### Reference

Inspired by [arnis](https://github.com/louis-e/arnis) — same OSM-to-game-world approach, adapted for Unity instead of Minecraft.

---

## 3. World Streaming

Loads Shenzhen around the player as they drive, managing memory and performance.

### Tile grid

- City divided into 256m x 256m tiles
- Each tile is a Unity Addressable scene containing road meshes, building meshes, and traffic graph data

### Loading rings

| Ring | Size | Content | Simulation |
|------|------|---------|------------|
| **Active** | 3x3 around player | Full meshes, colliders | Full physics, full traffic AI |
| **Buffer** | 5x5 | Full meshes, no colliders | Rail traffic (vehicles on splines) |
| **LOD** | 7x7 | Impostor meshes | Ghost traffic (lights only at night) |
| **Beyond** | — | Skybox silhouette | None |

### Streaming behavior

- Player position checked each frame against tile boundaries
- Crossing into a new tile shifts all rings: new tiles load async, far tiles unload
- Additive scene loading via `SceneManager.LoadSceneAsync` with `LoadSceneMode.Additive`
- Traffic AI vehicles are pooled — returned on tile unload, spawned on tile load

### Floating origin

Shenzhen spans ~80km. Unity float precision degrades far from origin. Periodically recenter the world origin to the player position, shifting all loaded objects.

### Memory budget

- Target ~2GB for loaded tiles at any time
- LOD tiles are lightweight (impostor cards, no collision)

---

## 4. Vehicle Physics

Custom raycast vehicle controller — Unity's built-in WheelCollider is insufficient for realistic sim.

### Suspension

- Per-wheel spring-damper system
- Rays cast downward from wheel anchors detect ground contact
- Spring force, damping, anti-roll bars — all configurable per-vehicle

### Tire model

- Pacejka "Magic Formula" for grip curves
- Separate lateral and longitudinal slip calculations
- Surface type (dry asphalt, wet road, painted lines) modifies grip coefficients

### Drivetrain

- Engine torque curve, gear ratios, differential
- Simplified but physically grounded — no arcade shortcuts

### Steering

- Speed-sensitive steering ratio
- Self-aligning torque from tire model — wheel naturally returns to center

### Weight transfer

- Dynamic load per wheel shifts under braking, acceleration, and cornering
- Affects grip per-wheel via tire model

### Player inputs

- Steering (analog stick or wheel), throttle, brake, handbrake
- Gear shift (auto or manual mode)
- Turn signals, headlights, wipers, horn
- Support for steering wheel peripherals (Logitech, Thrustmaster) via Unity Input System

### Surface interaction

- Road surface type from OSM pipeline tags per road segment
- Wet modifier from weather system reduces grip coefficients
- Painted road markings slightly more slippery when wet

### Cameras

- Interior (dashboard view with functional mirrors)
- Hood cam
- Chase cam
- Free look
- Head bob and sway tied to suspension movement in interior view

### Initial vehicle

- Mid-range sedan (BYD Qin or similar Shenzhen-common car)
- Tuned for predictable, forgiving handling
- Single vehicle for initial build; garage expansion later

---

## 5. Traffic AI

Dense, reactive traffic that makes Shenzhen feel real.

### Lane graph

- Directed graph from OSM pipeline: nodes are lane waypoints, edges are lane segments
- Encodes: speed limit, lane type (driving, bus, turn-only), signal group, yield rules
- Intersections are subgraphs with connection lanes (left turn, right turn, straight through)

### Traffic signals

- Signal controllers run phase cycles based on real Shenzhen timing (from OSM data, estimated otherwise)
- Phases: green, yellow, red, left-turn arrow, pedestrian walk
- AI vehicles query signal state at each intersection approach

### Vehicle AI — 3 behavior layers

1. **Path layer** — Origin/destination assigned per vehicle. A* on lane graph picks route. Destinations contextually appropriate to time of day (residential to commercial in morning, reverse in evening)
2. **Drive layer** — Follow lane centerline, maintain speed limit, decelerate for curves, stop at red signals, yield at merges. Intelligent Driver Model (IDM) for car-following and spacing
3. **React layer** — Respond to dynamic events: player cutting in, emergency stops, lane changes to pass slow vehicles, honking when blocked

### Density by time of day

| Time | Density | Behavior |
|------|---------|----------|
| Rush hour (7-9am, 5-7pm) | Maximum | Slow speeds, signal queuing |
| Midday | Moderate | Normal flow |
| Night (11pm-5am) | Sparse | Faster flow, fewer vehicles |
| Heavy weather | Slightly reduced | All periods |

### Pedestrians

- Spawn at crosswalks and sidewalks near intersections
- Wait for walk signals, cross in groups
- Simple avoidance — no free-roaming, just intersection crossings

### Performance LOD

| Level | Ring | Behavior |
|-------|------|----------|
| **Full sim** | Active (3x3) | Complete AI, physics collision |
| **Rail sim** | Buffer (5x5) | Follow lane splines at set speeds, no decisions |
| **Ghost sim** | Beyond | Headlight/taillight particles at night only |

### Pool budget

- ~200 fully simulated vehicles
- ~500 rail vehicles
- Unlimited ghost particles

---

## 6. Environment

Day-night cycle, weather, and the soundscape.

### Day-night cycle

- Configurable time scale (default: 1 real minute = 1 game hour, full cycle in 24 minutes)
- Player can set time manually from pause menu
- Sun/moon position from real Shenzhen latitude (22.5N) for accurate light angles
- Directional light color temperature shifts through golden hour, blue hour, night
- City lights (streetlamps, building windows, neon signs) activate at dusk via light probes and emissive triggers

### Weather system

- **States:** Clear, Overcast, Light Rain, Heavy Rain, Fog, Thunderstorm
- Gradual transitions (cloud cover builds before rain)
- Shenzhen-appropriate: subtropical climate, frequent rain, humid haze, no snow

#### Effects per state

| State | Visual | Physics | Audio |
|-------|--------|---------|-------|
| **Clear** | Sharp shadows, neon reflections | Baseline grip | City hum |
| **Overcast** | Flat lighting, grey sky | Baseline grip | Muted ambience |
| **Light Rain** | Rain particles, damp roads | Slight grip reduction | Light rain patter |
| **Heavy Rain** | Dense rain, wet road reflections, darkened asphalt | Significant grip reduction, wiper necessity | Heavy rain, splash |
| **Fog** | Distance fog + volumetric ground fog | Baseline grip, reduced visibility | Dampened sounds |
| **Thunderstorm** | Rain + lightning flashes | Heavy rain grip + gusts | Thunder, heavy rain |
| **Clear Night** | Neon reflects off dry roads, light pollution hides stars | Baseline grip | Quieter city, crickets |

### Audio — two layers

**Layer 1: Ambient soundscape (always on)**
- City hum shifts by district (dense urban vs highway vs coastal)
- Time-aware: construction daytime, crickets at night, distant karaoke from entertainment districts
- Weather sounds: rain intensity, thunder, wind
- Spatialized 3D traffic sounds from AI vehicles (engines, horns, tires)

**Layer 2: In-car radio (player-controlled)**

| Station | Genre |
|---------|-------|
| FM 88.1 | Lo-fi / chillhop instrumentals |
| FM 94.6 | Cantonese & Mandarin pop |
| FM 101.3 | Electronic / synthwave |
| FM 107.8 | Talk radio (city tips, fake ads, Shenzhen trivia) |

- Radio off by default — ambient soundscape is the baseline
- 2 stations available at start, remaining 2 unlocked via milestones
- Volume and station controlled from dashboard or hotkeys

---

## 7. Progression

Freeform exploration with quiet milestones.

### Stats tracked (always running)

- Total km driven
- Districts visited (Futian, Nanshan, Luohu, Bao'an, Longgang, etc.)
- Roads discovered (percentage of road network driven)
- Time spent driving at each time of day
- Weather conditions driven through
- Clean driving streak (km without collisions or violations)
- Traffic violations (red lights, speeding, wrong lane)
- Near-misses avoided

### Milestones

| Category | Examples |
|----------|---------|
| **Explorer** | "First 10km", "Visited 3 districts", "Found the coast road", "50% roads discovered" |
| **Night Owl** | "First midnight drive", "10 hours after dark", "Thunderstorm night drive" |
| **Clean Driver** | "100km no violations", "500km clean streak" |
| **Weather** | "First rain drive", "All weather types", "50km in heavy rain" |
| **City Knowledge** | "Found Huaqiangbei", "Crossed every bridge", "Full length of Shennan Road" |

### Milestone rewards

- New starting locations (spawn at different districts)
- Dashboard cosmetics (hanging ornaments, phone mounts, air fresheners)
- Radio station unlocks (start with 2, earn the other 2)
- Paint colors for the sedan

### Save system

- Single save file: JSON-backed ScriptableObject
- Stores: stats, unlocked milestones, car cosmetics, last position/time/weather, radio preferences
- Auto-saves on milestone unlock and every 5 minutes

### No fail states

Collisions cause visual damage (dents, cracked lights) and mechanical effects (alignment drift, engine stutter) but the player never "dies." Pull over and wait for slow recovery, or drive to a garage to reset. The game never punishes exploration.

---

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Engine | Unity (C#) | User preference, strong ecosystem for sim games |
| City generation | Offline pipeline (Rust) | Best runtime performance, predictable loading |
| Map data | OpenStreetMap + SRTM | Free, comprehensive, Shenzhen coverage is good |
| Vehicle physics | Custom raycast + Pacejka | WheelCollider insufficient for realistic sim |
| Traffic AI | Lane-graph + IDM + behavior layers | Scalable, realistic, performance-manageable |
| World streaming | Addressable tile scenes | Unity-native, async loading, memory-efficient |
| Input | Unity Input System | Supports gamepad + steering wheels |
| Rendering | URP (Universal Render Pipeline) | Good balance of quality and performance for sim |
| Audio | FMOD or Wwise | Professional spatial audio, radio system support |

---

## Scope & Constraints

- **Initial build:** One district (e.g., Futian CBD) fully playable, with LOD stubs for surrounding districts
- **Single vehicle** with deep physics tuning
- **Medium scope:** ~12-18 month development timeline for a vertical slice
- **Platform:** PC first (Windows), expandable to console
- **Target performance:** 60fps at 1080p on mid-range hardware
