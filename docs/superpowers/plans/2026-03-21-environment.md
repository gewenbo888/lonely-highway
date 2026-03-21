# Environment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the day-night cycle, weather system, and two-layer audio system (ambient soundscape + in-car radio) that makes Shenzhen feel alive.

**Architecture:** A `TimeManager` drives the game clock. A `WeatherManager` manages state transitions and spawns visual effects. A `LightingController` adjusts the directional light, skybox, and city light emissives based on time and weather. An `AmbientAudioManager` blends soundscape layers, and a `RadioSystem` handles in-car station playback. All systems publish state that other systems (vehicle physics grip, traffic density) can query.

**Tech Stack:** Unity 2022.3 LTS, URP, C#, Unity Audio (placeholder; FMOD/Wwise later), Unity Test Framework

**Spec reference:** `docs/superpowers/specs/2026-03-21-lonely-highway-design.md` — Section 6 (Environment)

---

## File Structure

```
Assets/
  LonelyHighway/
    Scripts/
      Environment/
        TimeManager.cs                — Game clock, time scale, sun/moon position
        WeatherManager.cs             — Weather state machine, transitions
        WeatherState.cs               — Weather enum and effect parameters
        LightingController.cs         — Directional light, skybox, city lights
        WetSurfaceController.cs       — Wet road shader parameters, grip modifier
        RainEffect.cs                 — Rain particle system controller
        FogController.cs              — Distance fog + volumetric fog
        LightningEffect.cs            — Lightning flash during thunderstorms
        AmbientAudioManager.cs        — Ambient soundscape layers
        RadioSystem.cs                — In-car radio stations
      Data/
        WeatherConfig.cs              — ScriptableObject: weather transition rules
        RadioStation.cs               — ScriptableObject: per-station data
        TimeConfig.cs                 — ScriptableObject: time scale, latitude
  Tests/
    EditMode/
      Environment/
        TimeManagerTests.cs
        WeatherManagerTests.cs
```

---

### Task 1: Assembly & Data Definitions

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Environment/LonelyHighway.Environment.asmdef`
- Create: `Assets/LonelyHighway/Scripts/Data/TimeConfig.cs`
- Create: `Assets/LonelyHighway/Scripts/Data/WeatherConfig.cs`
- Create: `Assets/LonelyHighway/Scripts/Data/RadioStation.cs`
- Create: `Assets/LonelyHighway/Scripts/Environment/WeatherState.cs`
- Create: `Assets/Tests/EditMode/Environment/EditModeEnvironmentTests.asmdef`

- [ ] **Step 1: Create assembly definitions**

`LonelyHighway.Environment.asmdef`:
```json
{
  "name": "LonelyHighway.Environment",
  "rootNamespace": "LonelyHighway.Environment",
  "references": ["LonelyHighway.Data", "LonelyHighway.Vehicle"],
  "includePlatforms": [],
  "autoReferenced": true
}
```

`EditModeEnvironmentTests.asmdef`:
```json
{
  "name": "EditModeEnvironmentTests",
  "rootNamespace": "LonelyHighway.Tests.EditMode.Environment",
  "references": ["LonelyHighway.Environment", "LonelyHighway.Data"],
  "includePlatforms": ["Editor"],
  "defineConstraints": ["UNITY_INCLUDE_TESTS"],
  "optionalUnityReferences": ["TestAssemblies"]
}
```

- [ ] **Step 2: Write data ScriptableObjects**

Write `TimeConfig.cs` (latitude, time scale defaults), `WeatherConfig.cs` (transition durations, weather probabilities), `RadioStation.cs` (station name, frequency, clips, locked status), and `WeatherState.cs` (enum + per-state parameter struct with grip multiplier, fog density, rain intensity, etc.).

- [ ] **Step 3: Commit**

```bash
git commit -m "feat: add environment assembly, time/weather/radio data definitions"
```

---

### Task 2: Time Manager

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Environment/TimeManager.cs`
- Create: `Assets/Tests/EditMode/Environment/TimeManagerTests.cs`

- [ ] **Step 1: Write tests** — verify time advances at correct scale, sun angle calculation for Shenzhen latitude, dawn/dusk detection

- [ ] **Step 2: Write TimeManager** — game clock with configurable scale (default 1 min = 1 hour), sun/moon position from latitude 22.5N, publishes `GameHour`, `SunAngle`, `IsDaytime`

- [ ] **Step 3: Run tests, commit**

```bash
git commit -m "feat: implement time manager with Shenzhen sun position"
```

---

### Task 3: Weather State Machine

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Environment/WeatherManager.cs`
- Create: `Assets/Tests/EditMode/Environment/WeatherManagerTests.cs`

- [ ] **Step 1: Write tests** — verify state transitions, transition durations, grip multiplier per state, random weather selection weighted by Shenzhen climate

- [ ] **Step 2: Write WeatherManager** — states (Clear, Overcast, LightRain, HeavyRain, Fog, Thunderstorm), gradual transitions, publishes `CurrentWeather`, `GripMultiplier`, `IsHeavyWeather`, `RainIntensity`

- [ ] **Step 3: Run tests, commit**

```bash
git commit -m "feat: implement weather state machine with gradual transitions"
```

---

### Task 4: Lighting Controller

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Environment/LightingController.cs`

- [ ] **Step 1: Write LightingController** — reads TimeManager + WeatherManager, adjusts: directional light intensity/color/angle, ambient light color, fog color, skybox exposure. City lights (emissive materials) toggle at dusk/dawn.

- [ ] **Step 2: Commit**

```bash
git commit -m "feat: implement dynamic lighting tied to time and weather"
```

---

### Task 5: Weather Visual Effects

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Environment/RainEffect.cs`
- Create: `Assets/LonelyHighway/Scripts/Environment/FogController.cs`
- Create: `Assets/LonelyHighway/Scripts/Environment/LightningEffect.cs`
- Create: `Assets/LonelyHighway/Scripts/Environment/WetSurfaceController.cs`

- [ ] **Step 1: Write RainEffect** — controls particle system emission rate based on `RainIntensity`
- [ ] **Step 2: Write FogController** — adjusts URP fog density and volumetric fog based on weather
- [ ] **Step 3: Write LightningEffect** — random flashes during thunderstorm, light intensity spike + audio
- [ ] **Step 4: Write WetSurfaceController** — sets wet road shader parameters (`_Wetness` material property), provides grip multiplier to vehicle physics

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: implement rain, fog, lightning, and wet surface effects"
```

---

### Task 6: Ambient Audio

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Environment/AmbientAudioManager.cs`

- [ ] **Step 1: Write AmbientAudioManager** — manages audio layers: city hum (crossfades by district), time-aware sounds (construction/crickets), weather sounds (rain/thunder/wind). Uses AudioSource per layer with volume crossfading.

- [ ] **Step 2: Commit**

```bash
git commit -m "feat: implement ambient audio manager with layered soundscape"
```

---

### Task 7: Radio System

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Environment/RadioSystem.cs`

- [ ] **Step 1: Write RadioSystem** — 4 stations (FM 88.1 lo-fi, FM 94.6 pop, FM 101.3 electronic, FM 107.8 talk). 2 unlocked at start, 2 locked. Player cycles stations, adjusts volume. Radio off by default. Plays AudioClip playlists per station.

- [ ] **Step 2: Commit**

```bash
git commit -m "feat: implement in-car radio system with 4 stations"
```

---

## Summary

| Task | Component | Tests |
|------|-----------|-------|
| 1 | Assembly + data types | — |
| 2 | Time manager | ~5 unit tests |
| 3 | Weather state machine | ~6 unit tests |
| 4 | Lighting controller | — |
| 5 | Weather visual effects | — |
| 6 | Ambient audio | — |
| 7 | Radio system | — |

**Total: 7 tasks, ~11 unit tests**
