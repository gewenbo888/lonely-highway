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
- **SRTM/ASTER elevation data** — base terrain height only (hills, slopes). Not used for vertical road positioning — that comes from OSM `layer`, `bridge`, and `tunnel` tags

### Processing stages

1. **Fetch** — Download OSM data for a bounding box covering Shenzhen (or specific districts)
2. **Parse** — Extract road centerlines, building polygons, signal positions, lane metadata
3. **Road mesh generation** — Extrude centerlines into road surfaces with lane widths, curbs, medians. Tag each segment with surface type (asphalt, concrete) for physics
4. **Elevated/tunnel handling** — Use OSM `layer=*`, `bridge=yes`, `tunnel=yes` tags to set vertical positioning. Elevated roads are placed at layer height (default 8m per layer). Tunnel roads are placed below ground with portal geometry. Ramp connections between levels use OSM link ways. Colliders generated for each level independently so vehicles can drive under overpasses
5. **Building shell generation** — Extrude footprints to height. Classify by OSM tags (residential, commercial, industrial) for facade variation
6. **Traffic graph generation** — Build a directed lane graph with connections, signal phases, speed limits. This is what Traffic AI pathfinds on. Multi-level interchanges produce separate subgraphs per layer, connected by ramp edges
7. **Tile chunking** — Divide the city into a grid (512m x 512m tiles). Each tile becomes a Unity scene with road meshes, building meshes, and traffic graph data as ScriptableObjects. Features crossing tile boundaries are duplicated into both tiles with shared edge IDs for seam stitching
8. **Export** — glTF meshes + ScriptableObjects + tile metadata JSON

### Data fallback strategy

OSM coverage in Shenzhen is uneven. The pipeline applies defaults when tags are missing:

| Missing data | Fallback |
|-------------|----------|
| Lane count | Infer from `highway` classification: `primary` = 3 lanes/dir, `secondary` = 2, `tertiary` = 1 |
| Speed limit | Infer from road class: `motorway` = 100, `primary` = 60, `secondary` = 40, `residential` = 30 (km/h) |
| Building height | Infer from land use zone: commercial = 40m, residential = 25m, industrial = 12m |
| Surface type | Default asphalt for all roads |
| Signal timing | Estimate from intersection complexity: 2-way = 60s cycle, 4-way = 90s, complex = 120s |

### Language

Rust (consistent with the arnis reference project).

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

- City divided into 512m x 512m tiles (larger than typical to reduce tile-crossing frequency at highway speeds)
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

### Tile boundary stitching

Features (roads, buildings) that cross tile boundaries are duplicated into both tiles during the pipeline's chunking stage. Each duplicate carries a shared edge ID. At runtime, the streaming system matches edge IDs between adjacent loaded tiles and snaps vertices to eliminate seams. Traffic graph edges crossing tile boundaries use the same shared ID mechanism for seamless AI pathfinding.

### Floating origin

Shenzhen spans ~80km. Unity float precision degrades far from origin. Recenter the world origin when the player moves more than 2km from current origin. Recentering shifts all loaded GameObjects in a single frame during `LateUpdate`, after physics has resolved. Performed only when no async tile loads are in flight to avoid race conditions.

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

- Interior (dashboard view with mirrors — rendered at half resolution, updated at 30fps to manage performance cost of 3 extra camera renders)
- Hood cam
- Chase cam
- Free look
- Head bob and sway tied to suspension movement in interior view

### Collision & damage model

- Collisions detected via Unity's physics collision callbacks on the vehicle body collider
- **Visual damage:** Deformation mesh vertices displaced at impact point, cracked light textures swap, paint scratches. Damage state stored as a per-panel health value (0-100%)
- **Mechanical damage:** Alignment drift (steering pulls), engine stutter (random torque drops), suspension sag. Each maps to a physics parameter modifier
- **Recovery:** Slow passive recovery over time (alignment self-corrects over ~5 minutes of driving). Full instant repair at garage locations
- **Garages:** Fixed map locations marked in OSM data (tagged `shop=car_repair` or manually placed). Appear on minimap. Player drives in, brief fade-to-black, vehicle fully repaired. No cost — garages are a convenience, not a punishment

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
- Intersections are subgraphs with connection lanes (left turn, right turn, straight through, U-turn)
- Highway on/off ramps encoded as yield-merge edges with gap-acceptance behavior
- Roundabouts encoded as circular one-way subgraphs with yield-on-entry

### Traffic signals

- Signal controllers run phase cycles based on real Shenzhen timing (from OSM data, estimated otherwise)
- Phases: green, yellow, red, left-turn arrow, pedestrian walk
- AI vehicles query signal state at each intersection approach

### Vehicle AI — 3 behavior layers

1. **Path layer** — Origin/destination assigned per vehicle. A* on lane graph picks route. Destinations contextually appropriate to time of day (residential to commercial in morning, reverse in evening)
2. **Drive layer** — Follow lane centerline, maintain speed limit, decelerate for curves, stop at red signals, yield at merges. Intelligent Driver Model (IDM) for car-following and spacing
3. **React layer** — Respond to dynamic events: player cutting in, emergency stops, lane changes to pass slow vehicles, honking when blocked

### Special vehicle types

- **Buses:** Follow designated bus lanes, stop at bus stops (pull over, pause 15-30s, re-merge). Routes derived from OSM `route=bus` relations where available
- **E-bikes/scooters:** High-frequency, lower speed, weave between lanes. Spawn on bike lanes and road edges. Significant presence in Shenzhen traffic — adds realism and challenge
- Emergency vehicles (ambulance, police) are out of scope for initial build

### Density by time of day

Base density values, modified by a weather multiplier (heavy rain/fog = 0.8x):

| Time | Density | Behavior |
|------|---------|----------|
| Rush hour (7-9am, 5-7pm) | Maximum | Slow speeds, signal queuing |
| Midday | Moderate | Normal flow |
| Night (11pm-5am) | Sparse | Faster flow, fewer vehicles |

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

- ~300 fully simulated vehicles (tuned to support ~30-50 vehicles visible at major intersections, across 9 active tiles)
- ~500 rail vehicles
- Unlimited ghost particles
- Budget is a tuning target — profiling during development will determine final numbers

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
| FM 107.8 | Talk radio (city tips, fake ads, Shenzhen trivia) — requires voice content production; placeholder static/music for initial build |

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

## 8. UI / HUD

### In-game HUD (minimal, sim-appropriate)

- **Speedometer** — analog gauge in dashboard (interior cam) or small digital overlay (other cams), showing km/h
- **Tachometer** — RPM gauge, visible in interior cam dashboard
- **Gear indicator** — current gear (or "A" for auto mode)
- **Turn signal indicators** — left/right arrows, visible in all camera modes
- **Minimap** — small corner map showing nearby roads, player position, discovered/undiscovered areas. Garage locations marked. Togglable
- **Milestone notification** — subtle toast notification when a milestone is unlocked, fades after 3 seconds
- **Radio display** — station name and frequency, shown briefly on station change

### Menus

- **Pause menu** — Resume, Map (full-screen discovered road map), Stats, Milestones, Settings, Quit
- **Settings** — Graphics (resolution, quality presets, mirror quality), Audio (master, ambient, radio, engine volumes), Controls (sensitivity, deadzone, peripheral mapping), Gameplay (time scale, units km/mi)
- **Garage screen** — Cosmetics selection (paint, dashboard items), damage status, repair button

### Design principle

HUD is minimal by default. Interior cam shows dashboard instruments; external cams show only a small speed readout and minimap. No clutter — the city is the focus.

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
| Pipeline language | Rust | Consistent with arnis reference, performant for mesh generation |

---

## Scope & Constraints

- **Initial build:** One district (e.g., Futian CBD) fully playable, with LOD stubs for surrounding districts
- **Single vehicle** with deep physics tuning
- **Medium scope:** ~12-18 month development timeline for a vertical slice
- **Platform:** PC first (Windows), expandable to console
- **Target performance:** 60fps at 1080p on mid-range hardware
- **Pipeline integration:** Rust tool runs outside Unity, outputs to an `Assets/GeneratedTiles/` directory. Unity Editor script auto-imports on refresh. Pipeline is run per-district during development, full city for release builds
- **Estimated generated data:** ~2-4 GB for full Shenzhen (meshes + traffic graphs + metadata)

### Out of scope (initial build)

- Multiplayer
- On-foot gameplay / character models
- Interior building exploration
- Vehicle purchasing / economy
- Emergency vehicles
- Motorcycles or vehicle classes beyond the sedan
- Mobile platform
- Voice-acted radio content (placeholder music/static for talk radio initially)

### URP rendering notes

Required URP features for visual targets:
- Screen-space reflections (wet road surfaces)
- Volumetric fog (fog weather state, night haze)
- Emissive materials at scale (building windows, neon signs)
- All confirmed available in URP 14+ (Unity 2022.3 LTS or later)
