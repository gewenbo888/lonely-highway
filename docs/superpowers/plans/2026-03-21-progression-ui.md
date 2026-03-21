# Progression & UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the stat tracking, milestone system, save/load, and all UI — HUD, minimap, menus, pause screen, garage screen, and milestone notifications.

**Architecture:** A `StatsTracker` accumulates driving statistics each frame. A `MilestoneManager` checks stat thresholds and unlocks rewards. A `SaveManager` persists state to JSON with atomic writes. UI uses Unity UI Toolkit (or legacy Canvas) with a minimal HUD, a fog-of-war minimap, and menu screens for pause/settings/garage.

**Tech Stack:** Unity 2022.3 LTS, C#, Unity UI Toolkit or Canvas, Unity Test Framework

**Spec reference:** `docs/superpowers/specs/2026-03-21-lonely-highway-design.md` — Sections 7 (Progression) and 8 (UI/HUD)

---

## File Structure

```
Assets/
  LonelyHighway/
    Scripts/
      Progression/
        StatsTracker.cs               — Accumulates all driving stats
        MilestoneManager.cs           — Checks thresholds, unlocks rewards
        MilestoneDefinition.cs        — ScriptableObject per milestone
        SaveManager.cs                — JSON save/load with atomic writes
        SaveData.cs                   — Serializable save state
      UI/
        HUDController.cs              — Main HUD: speedometer, gear, signals, radio display
        MinimapController.cs          — Corner minimap with fog-of-war
        MilestoneNotification.cs      — Toast popup on milestone unlock
        PauseMenuController.cs        — Pause menu with subpanels
        SettingsPanel.cs              — Graphics, audio, controls, gameplay settings
        GaragePanel.cs                — Cosmetics and repair UI
        FullMapPanel.cs               — Full-screen discovered road map
      Data/
        MilestoneDatabase.cs          — ScriptableObject: all milestone definitions
  Tests/
    EditMode/
      Progression/
        StatsTrackerTests.cs
        MilestoneManagerTests.cs
        SaveManagerTests.cs
```

---

### Task 1: Assembly & Save Data Types

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Progression/LonelyHighway.Progression.asmdef`
- Create: `Assets/LonelyHighway/Scripts/Progression/SaveData.cs`
- Create: `Assets/LonelyHighway/Scripts/UI/LonelyHighway.UI.asmdef`
- Create: `Assets/Tests/EditMode/Progression/EditModeProgressionTests.asmdef`

- [ ] **Step 1: Create assembly definitions**

- [ ] **Step 2: Write SaveData** — serializable struct with all stat fields, milestone unlock list, cosmetic selections, last position/time/weather, radio prefs

- [ ] **Step 3: Commit**

```bash
git commit -m "feat: add progression/UI assemblies and save data types"
```

---

### Task 2: Stats Tracker

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Progression/StatsTracker.cs`
- Create: `Assets/Tests/EditMode/Progression/StatsTrackerTests.cs`

- [ ] **Step 1: Write tests** — km accumulation, district visit tracking, clean driving streak (resets on violation), violation counting, time-of-day tracking

- [ ] **Step 2: Write StatsTracker** — reads VehicleController each frame, accumulates: totalKm, districtsVisited (HashSet), roadsDiscovered (percentage), timeByPeriod, weathersDriven, cleanStreak, violations, nearMisses

- [ ] **Step 3: Run tests, commit**

```bash
git commit -m "feat: implement stats tracker for driving statistics"
```

---

### Task 3: Milestone System

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Progression/MilestoneDefinition.cs`
- Create: `Assets/LonelyHighway/Scripts/Progression/MilestoneManager.cs`
- Create: `Assets/LonelyHighway/Scripts/Data/MilestoneDatabase.cs`
- Create: `Assets/Tests/EditMode/Progression/MilestoneManagerTests.cs`

- [ ] **Step 1: Write MilestoneDefinition** — ScriptableObject with: id, name, description, category, stat type, threshold value, reward type (spawn point / cosmetic / radio / paint)

- [ ] **Step 2: Write tests** — milestone unlocks when stat exceeds threshold, already-unlocked milestones don't re-trigger, multiple milestones can unlock in one frame

- [ ] **Step 3: Write MilestoneManager** — checks StatsTracker against MilestoneDatabase each second, fires UnlockEvent, tracks unlocked set

- [ ] **Step 4: Run tests, commit**

```bash
git commit -m "feat: implement milestone system with threshold-based unlocks"
```

---

### Task 4: Save Manager

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Progression/SaveManager.cs`
- Create: `Assets/Tests/EditMode/Progression/SaveManagerTests.cs`

- [ ] **Step 1: Write tests** — save creates JSON file, load restores correct values, atomic write (temp file → rename), .bak file created, auto-save timer fires at 5-minute intervals

- [ ] **Step 2: Write SaveManager** — serializes SaveData to JSON, writes to temp file then renames (atomic), keeps .bak of previous save. Auto-saves every 5 min and on milestone unlock. Load on startup.

- [ ] **Step 3: Run tests, commit**

```bash
git commit -m "feat: implement save manager with atomic writes and backup"
```

---

### Task 5: HUD

**Files:**
- Create: `Assets/LonelyHighway/Scripts/UI/HUDController.cs`

- [ ] **Step 1: Write HUDController** — reads VehicleController for speed/RPM/gear, shows: digital speedometer (km/h), gear indicator ("1"-"6" or "A"), turn signal arrows, radio station name on change (fades after 3s). Minimal overlay for external cams, full dashboard for interior cam.

- [ ] **Step 2: Commit**

```bash
git commit -m "feat: implement minimal HUD with speedometer, gear, signals"
```

---

### Task 6: Minimap

**Files:**
- Create: `Assets/LonelyHighway/Scripts/UI/MinimapController.cs`

- [ ] **Step 1: Write MinimapController** — corner minimap rendered from pre-baked tile textures (from pipeline minimap.png). Runtime fog-of-war mask: RenderTexture that reveals pixels as player drives within radius. Shows player arrow, garage icons. Togglable.

- [ ] **Step 2: Commit**

```bash
git commit -m "feat: implement minimap with fog-of-war discovery"
```

---

### Task 7: Milestone Notification

**Files:**
- Create: `Assets/LonelyHighway/Scripts/UI/MilestoneNotification.cs`

- [ ] **Step 1: Write MilestoneNotification** — listens to MilestoneManager.OnUnlock, shows toast panel (milestone name + icon) that slides in, holds 3 seconds, fades out. Queue system if multiple milestones unlock at once.

- [ ] **Step 2: Commit**

```bash
git commit -m "feat: implement milestone toast notification system"
```

---

### Task 8: Pause Menu & Settings

**Files:**
- Create: `Assets/LonelyHighway/Scripts/UI/PauseMenuController.cs`
- Create: `Assets/LonelyHighway/Scripts/UI/SettingsPanel.cs`
- Create: `Assets/LonelyHighway/Scripts/UI/FullMapPanel.cs`
- Create: `Assets/LonelyHighway/Scripts/UI/GaragePanel.cs`

- [ ] **Step 1: Write PauseMenuController** — ESC toggles pause, time scale 0, shows: Resume, Map, Stats, Milestones, Settings, Quit buttons

- [ ] **Step 2: Write SettingsPanel** — Graphics (resolution dropdown, quality preset, mirror toggle), Audio (sliders: master, ambient, radio, engine), Controls (sensitivity, deadzone), Gameplay (time scale, km/mi toggle)

- [ ] **Step 3: Write FullMapPanel** — full-screen map view, assembled from minimap tiles, shows discovered roads, garage locations, districts

- [ ] **Step 4: Write GaragePanel** — opened when entering garage trigger. Shows: paint color selector, dashboard cosmetic slots, damage status per panel, repair button

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: implement pause menu, settings, full map, and garage UI"
```

---

## Summary

| Task | Component | Tests |
|------|-----------|-------|
| 1 | Assembly + save data | — |
| 2 | Stats tracker | ~6 unit tests |
| 3 | Milestone system | ~4 unit tests |
| 4 | Save manager | ~5 unit tests |
| 5 | HUD | — |
| 6 | Minimap | — |
| 7 | Milestone notification | — |
| 8 | Pause menu + settings + garage | — |

**Total: 8 tasks, ~15 unit tests**
