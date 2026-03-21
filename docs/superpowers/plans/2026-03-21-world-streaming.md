# World Streaming Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Unity tile streaming system that loads/unloads Shenzhen around the player as they drive, with three LOD rings, floating origin, and boundary stitching.

**Architecture:** A `WorldStreamer` MonoBehaviour tracks the player's tile position. When the player crosses a tile boundary, it computes which tiles should be in each ring (Active 3x3, Buffer 5x5, LOD 7x7), async-loads new tiles via Addressables, and unloads tiles that fall outside the outer ring. A `FloatingOrigin` component periodically recenters the world. A `TileData` ScriptableObject per tile holds metadata imported from the pipeline output.

**Tech Stack:** Unity 2022.3 LTS, Unity Addressables, C#, Unity Test Framework

**Spec reference:** `docs/superpowers/specs/2026-03-21-lonely-highway-design.md` — Section 3 (World Streaming)

**Dependency:** Requires pipeline output (Plan 2) for real data, but can be developed and tested with procedural placeholder tiles.

---

## File Structure

```
Assets/
  LonelyHighway/
    Scripts/
      Streaming/
        WorldStreamer.cs              — Main streaming controller, ring management
        TileGrid.cs                   — Tile coordinate math, ring computation
        TileLoader.cs                 — Async tile loading/unloading via Addressables
        TileState.cs                  — Per-tile runtime state (loading, loaded, unloading)
        FloatingOrigin.cs             — World origin recentering
        BoundaryStitcher.cs           — Seam stitching between adjacent tiles
        TileImporter.cs               — Editor script: imports pipeline output into Unity
      Data/
        TileMetadata.cs               — ScriptableObject: per-tile bounds, counts, signals
        TrafficGraphData.cs           — ScriptableObject: serialized lane graph per tile
    Data/
      TileDatabase.asset              — ScriptableObject: index of all tile metadata
  Tests/
    EditMode/
      Streaming/
        TileGridTests.cs              — Unit tests for tile coordinate math
        FloatingOriginTests.cs        — Unit tests for recentering logic
    PlayMode/
      Streaming/
        WorldStreamerTests.cs          — Integration tests with placeholder tiles
```

---

### Task 1: Assembly Definition & Data Types

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Streaming/LonelyHighway.Streaming.asmdef`
- Create: `Assets/LonelyHighway/Scripts/Data/TileMetadata.cs`
- Create: `Assets/LonelyHighway/Scripts/Data/TrafficGraphData.cs`
- Create: `Assets/Tests/EditMode/Streaming/EditModeStreamingTests.asmdef`

- [ ] **Step 1: Create streaming assembly definition**

```json
{
  "name": "LonelyHighway.Streaming",
  "rootNamespace": "LonelyHighway.Streaming",
  "references": ["LonelyHighway.Data", "LonelyHighway.Vehicle", "Unity.Addressables", "Unity.ResourceManager"],
  "includePlatforms": [],
  "autoReferenced": true
}
```

- [ ] **Step 2: Create test assembly definition**

```json
{
  "name": "EditModeStreamingTests",
  "rootNamespace": "LonelyHighway.Tests.EditMode.Streaming",
  "references": ["LonelyHighway.Streaming", "LonelyHighway.Data"],
  "includePlatforms": ["Editor"],
  "defineConstraints": ["UNITY_INCLUDE_TESTS"],
  "optionalUnityReferences": ["TestAssemblies"]
}
```

- [ ] **Step 3: Write TileMetadata ScriptableObject**

```csharp
// Assets/LonelyHighway/Scripts/Data/TileMetadata.cs
using UnityEngine;

namespace LonelyHighway.Data
{
    [CreateAssetMenu(fileName = "NewTileMetadata", menuName = "LonelyHighway/Tile Metadata")]
    public class TileMetadata : ScriptableObject
    {
        public int coordX;
        public int coordY;

        [Header("Bounds (world meters)")]
        public float minX;
        public float minZ;
        public float maxX;
        public float maxZ;

        [Header("Content counts")]
        public int roadMeshCount;
        public int buildingMeshCount;
        public int trafficNodeCount;
        public int trafficEdgeCount;

        [Header("Signals")]
        public Vector2[] signalPositions;

        [Header("Addressable references")]
        public string sceneAddress;
        public string lodSceneAddress;
    }
}
```

- [ ] **Step 4: Write TrafficGraphData ScriptableObject**

```csharp
// Assets/LonelyHighway/Scripts/Data/TrafficGraphData.cs
using UnityEngine;
using System;

namespace LonelyHighway.Data
{
    [CreateAssetMenu(fileName = "NewTrafficGraph", menuName = "LonelyHighway/Traffic Graph")]
    public class TrafficGraphData : ScriptableObject
    {
        public LaneNode[] nodes;
        public LaneEdge[] edges;
        public SignalController[] signals;

        [Serializable]
        public struct LaneNode
        {
            public int id;
            public float x;
            public float z;
            public float y;
        }

        [Serializable]
        public struct LaneEdge
        {
            public int id;
            public int fromNode;
            public int toNode;
            public float speedLimitKmh;
            public int laneIndex;
            public long roadId;
            public int layer;
            public float length;
            public int sharedEdgeId; // -1 if not a boundary edge
        }

        [Serializable]
        public struct SignalController
        {
            public long id;
            public float x;
            public float z;
            public float cycleTime;
            public int[] controlledEdgeIds;
        }
    }
}
```

- [ ] **Step 5: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Streaming/ Assets/LonelyHighway/Scripts/Data/TileMetadata.cs Assets/LonelyHighway/Scripts/Data/TrafficGraphData.cs Assets/Tests/EditMode/Streaming/
git commit -m "feat: add streaming assembly, tile metadata, and traffic graph data types"
```

---

### Task 2: Tile Grid Math

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Streaming/TileGrid.cs`
- Create: `Assets/Tests/EditMode/Streaming/TileGridTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Streaming/TileGridTests.cs
using NUnit.Framework;
using LonelyHighway.Streaming;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode.Streaming
{
    public class TileGridTests
    {
        private const float TileSize = 512f;

        [Test]
        public void WorldToTile_Origin_ReturnsZeroZero()
        {
            var coord = TileGrid.WorldToTile(Vector3.zero, TileSize);
            Assert.AreEqual(0, coord.x);
            Assert.AreEqual(0, coord.y);
        }

        [Test]
        public void WorldToTile_InsideFirstTile_ReturnsZeroZero()
        {
            var coord = TileGrid.WorldToTile(new Vector3(100f, 0f, 200f), TileSize);
            Assert.AreEqual(0, coord.x);
            Assert.AreEqual(0, coord.y);
        }

        [Test]
        public void WorldToTile_CrossesBoundary_ReturnsNextTile()
        {
            var coord = TileGrid.WorldToTile(new Vector3(600f, 0f, 600f), TileSize);
            Assert.AreEqual(1, coord.x);
            Assert.AreEqual(1, coord.y);
        }

        [Test]
        public void WorldToTile_NegativeCoords()
        {
            var coord = TileGrid.WorldToTile(new Vector3(-100f, 0f, -200f), TileSize);
            Assert.AreEqual(-1, coord.x);
            Assert.AreEqual(-1, coord.y);
        }

        [Test]
        public void GetRingTiles_3x3_Returns9Tiles()
        {
            var center = new Vector2Int(5, 5);
            var tiles = TileGrid.GetRingTiles(center, 1);
            Assert.AreEqual(9, tiles.Length);
        }

        [Test]
        public void GetRingTiles_5x5_Returns25Tiles()
        {
            var center = new Vector2Int(5, 5);
            var tiles = TileGrid.GetRingTiles(center, 2);
            Assert.AreEqual(25, tiles.Length);
        }

        [Test]
        public void GetRingTiles_ContainsCenter()
        {
            var center = new Vector2Int(3, 7);
            var tiles = TileGrid.GetRingTiles(center, 1);
            Assert.Contains(center, tiles);
        }

        [Test]
        public void TileBounds_CorrectForTile()
        {
            var bounds = TileGrid.GetTileBounds(new Vector2Int(2, 3), TileSize);
            Assert.AreEqual(1024f, bounds.min.x, 0.01f);
            Assert.AreEqual(1536f, bounds.min.y, 0.01f);
            Assert.AreEqual(1536f, bounds.max.x, 0.01f);
            Assert.AreEqual(2048f, bounds.max.y, 0.01f);
        }

        [Test]
        public void RingDifference_NewTilesOnly()
        {
            var oldCenter = new Vector2Int(5, 5);
            var newCenter = new Vector2Int(6, 5);
            var oldTiles = TileGrid.GetRingTiles(oldCenter, 1);
            var newTiles = TileGrid.GetRingTiles(newCenter, 1);

            var toLoad = TileGrid.TileDifference(newTiles, oldTiles);
            var toUnload = TileGrid.TileDifference(oldTiles, newTiles);

            Assert.Greater(toLoad.Length, 0);
            Assert.Greater(toUnload.Length, 0);
            Assert.AreEqual(toLoad.Length, toUnload.Length); // Symmetric for 1-tile shift
        }
    }
}
```

- [ ] **Step 2: Write TileGrid implementation**

```csharp
// Assets/LonelyHighway/Scripts/Streaming/TileGrid.cs
using UnityEngine;
using System.Collections.Generic;
using System.Linq;

namespace LonelyHighway.Streaming
{
    public static class TileGrid
    {
        public static Vector2Int WorldToTile(Vector3 worldPos, float tileSize)
        {
            return new Vector2Int(
                Mathf.FloorToInt(worldPos.x / tileSize),
                Mathf.FloorToInt(worldPos.z / tileSize)
            );
        }

        /// <summary>
        /// Get all tile coordinates within a square ring of given radius around center.
        /// Radius 1 = 3x3, radius 2 = 5x5, radius 3 = 7x7.
        /// </summary>
        public static Vector2Int[] GetRingTiles(Vector2Int center, int radius)
        {
            var tiles = new List<Vector2Int>();
            for (int x = -radius; x <= radius; x++)
            {
                for (int y = -radius; y <= radius; y++)
                {
                    tiles.Add(new Vector2Int(center.x + x, center.y + y));
                }
            }
            return tiles.ToArray();
        }

        /// <summary>
        /// Get world-space bounds for a tile.
        /// Returns (min, max) as Vector2 (x = world X, y = world Z).
        /// </summary>
        public static (Vector2 min, Vector2 max) GetTileBounds(Vector2Int coord, float tileSize)
        {
            return (
                new Vector2(coord.x * tileSize, coord.y * tileSize),
                new Vector2((coord.x + 1) * tileSize, (coord.y + 1) * tileSize)
            );
        }

        /// <summary>
        /// Returns tiles in 'a' that are not in 'b'.
        /// </summary>
        public static Vector2Int[] TileDifference(Vector2Int[] a, Vector2Int[] b)
        {
            var bSet = new HashSet<Vector2Int>(b);
            return a.Where(t => !bSet.Contains(t)).ToArray();
        }
    }
}
```

- [ ] **Step 3: Run tests**

Run: Test Runner → EditMode → TileGridTests
Expected: All 9 PASS

- [ ] **Step 4: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Streaming/TileGrid.cs Assets/Tests/EditMode/Streaming/TileGridTests.cs
git commit -m "feat: implement tile grid coordinate math with ring computation"
```

---

### Task 3: Tile State Machine

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Streaming/TileState.cs`

- [ ] **Step 1: Write TileState**

```csharp
// Assets/LonelyHighway/Scripts/Streaming/TileState.cs
using UnityEngine;
using UnityEngine.ResourceManagement.AsyncOperations;

namespace LonelyHighway.Streaming
{
    public enum TileLoadState
    {
        Unloaded,
        Loading,
        Loaded,
        Unloading,
    }

    public enum TileRing
    {
        None,
        Active,   // 3x3 — full physics + traffic AI
        Buffer,   // 5x5 — meshes, rail traffic
        LOD,      // 7x7 — impostors, ghost lights
    }

    public class TileState
    {
        public Vector2Int Coord { get; }
        public TileLoadState LoadState { get; set; }
        public TileRing Ring { get; set; }
        public AsyncOperationHandle? LoadHandle { get; set; }
        public GameObject SceneRoot { get; set; }

        public TileState(Vector2Int coord)
        {
            Coord = coord;
            LoadState = TileLoadState.Unloaded;
            Ring = TileRing.None;
        }

        /// <summary>
        /// Determine which ring this tile belongs to relative to the player's current tile.
        /// </summary>
        public static TileRing DetermineRing(Vector2Int tileCoord, Vector2Int playerTile)
        {
            int dx = Mathf.Abs(tileCoord.x - playerTile.x);
            int dy = Mathf.Abs(tileCoord.y - playerTile.y);
            int maxDist = Mathf.Max(dx, dy);

            if (maxDist <= 1) return TileRing.Active;
            if (maxDist <= 2) return TileRing.Buffer;
            if (maxDist <= 3) return TileRing.LOD;
            return TileRing.None;
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Streaming/TileState.cs
git commit -m "feat: add tile state machine with ring classification"
```

---

### Task 4: Tile Loader

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Streaming/TileLoader.cs`

- [ ] **Step 1: Write TileLoader**

```csharp
// Assets/LonelyHighway/Scripts/Streaming/TileLoader.cs
using UnityEngine;
using UnityEngine.AddressableAssets;
using UnityEngine.ResourceManagement.AsyncOperations;
using System;
using System.Collections.Generic;

namespace LonelyHighway.Streaming
{
    public class TileLoader : MonoBehaviour
    {
        private readonly Dictionary<Vector2Int, TileState> _tiles = new();

        public int LoadingCount => CountByState(TileLoadState.Loading);
        public int LoadedCount => CountByState(TileLoadState.Loaded);
        public bool IsAnyLoading => LoadingCount > 0;

        public event Action<Vector2Int, TileRing> OnTileLoaded;
        public event Action<Vector2Int> OnTileUnloaded;

        /// <summary>
        /// Request a tile to be loaded at the given ring level.
        /// </summary>
        public void RequestLoad(Vector2Int coord, TileRing ring, string sceneAddress)
        {
            if (_tiles.TryGetValue(coord, out var existing))
            {
                existing.Ring = ring;
                if (existing.LoadState == TileLoadState.Loaded || existing.LoadState == TileLoadState.Loading)
                    return;
            }

            var state = new TileState(coord) { Ring = ring, LoadState = TileLoadState.Loading };

            var handle = Addressables.LoadSceneAsync(sceneAddress, UnityEngine.SceneManagement.LoadSceneMode.Additive);
            state.LoadHandle = handle;

            handle.Completed += op =>
            {
                if (op.Status == AsyncOperationStatus.Succeeded)
                {
                    state.LoadState = TileLoadState.Loaded;
                    OnTileLoaded?.Invoke(coord, ring);
                    log($"Tile {coord} loaded (ring: {ring})");
                }
                else
                {
                    state.LoadState = TileLoadState.Unloaded;
                    log($"Tile {coord} failed to load: {op.OperationException}");
                }
            };

            _tiles[coord] = state;
        }

        /// <summary>
        /// Request a tile to be unloaded.
        /// </summary>
        public void RequestUnload(Vector2Int coord)
        {
            if (!_tiles.TryGetValue(coord, out var state))
                return;

            if (state.LoadState != TileLoadState.Loaded)
                return;

            state.LoadState = TileLoadState.Unloading;

            if (state.LoadHandle.HasValue)
            {
                Addressables.UnloadSceneAsync(state.LoadHandle.Value).Completed += op =>
                {
                    state.LoadState = TileLoadState.Unloaded;
                    _tiles.Remove(coord);
                    OnTileUnloaded?.Invoke(coord);
                    log($"Tile {coord} unloaded");
                };
            }
        }

        /// <summary>
        /// Update the ring classification for an already-loaded tile.
        /// This affects what simulation runs on it (full AI vs rail vs ghost).
        /// </summary>
        public void UpdateRing(Vector2Int coord, TileRing newRing)
        {
            if (_tiles.TryGetValue(coord, out var state))
                state.Ring = newRing;
        }

        public TileState GetTileState(Vector2Int coord)
        {
            return _tiles.TryGetValue(coord, out var state) ? state : null;
        }

        public IEnumerable<TileState> GetLoadedTiles()
        {
            foreach (var kvp in _tiles)
            {
                if (kvp.Value.LoadState == TileLoadState.Loaded)
                    yield return kvp.Value;
            }
        }

        private int CountByState(TileLoadState target)
        {
            int count = 0;
            foreach (var kvp in _tiles)
                if (kvp.Value.LoadState == target) count++;
            return count;
        }

        private void log(string msg)
        {
            Debug.Log($"[TileLoader] {msg}");
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Streaming/TileLoader.cs
git commit -m "feat: implement async tile loader with Addressables"
```

---

### Task 5: Floating Origin

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Streaming/FloatingOrigin.cs`
- Create: `Assets/Tests/EditMode/Streaming/FloatingOriginTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Streaming/FloatingOriginTests.cs
using NUnit.Framework;
using LonelyHighway.Streaming;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode.Streaming
{
    public class FloatingOriginTests
    {
        [Test]
        public void ShouldRecenter_WhenBeyondThreshold_ReturnsTrue()
        {
            var origin = Vector3.zero;
            var playerPos = new Vector3(2500f, 0f, 0f);
            bool should = FloatingOrigin.ShouldRecenter(playerPos, origin, 2000f);
            Assert.IsTrue(should);
        }

        [Test]
        public void ShouldRecenter_WhenWithinThreshold_ReturnsFalse()
        {
            var origin = Vector3.zero;
            var playerPos = new Vector3(1000f, 0f, 500f);
            bool should = FloatingOrigin.ShouldRecenter(playerPos, origin, 2000f);
            Assert.IsFalse(should);
        }

        [Test]
        public void CalculateShift_ReturnsPlayerPosition()
        {
            var playerPos = new Vector3(3000f, 5f, 4000f);
            var shift = FloatingOrigin.CalculateShift(playerPos);
            // Shift should bring player back to near origin (only XZ, ignore Y)
            Assert.AreEqual(-3000f, shift.x, 0.01f);
            Assert.AreEqual(0f, shift.y, 0.01f);
            Assert.AreEqual(-4000f, shift.z, 0.01f);
        }
    }
}
```

- [ ] **Step 2: Write FloatingOrigin**

```csharp
// Assets/LonelyHighway/Scripts/Streaming/FloatingOrigin.cs
using UnityEngine;

namespace LonelyHighway.Streaming
{
    public class FloatingOrigin : MonoBehaviour
    {
        [Tooltip("Distance from origin that triggers recentering (meters)")]
        public float recenterThreshold = 2000f;

        [Tooltip("Reference to the tile loader to check for in-flight loads")]
        public TileLoader tileLoader;

        private Transform _player;
        private Vector3 _currentOriginOffset;

        /// <summary>
        /// Total accumulated origin offset. Add this to any GPS/world coordinate
        /// to get the current Unity position.
        /// </summary>
        public Vector3 OriginOffset => _currentOriginOffset;

        public void Initialize(Transform player)
        {
            _player = player;
        }

        private void LateUpdate()
        {
            if (_player == null) return;

            // Don't recenter while tiles are loading (race condition)
            if (tileLoader != null && tileLoader.IsAnyLoading) return;

            if (ShouldRecenter(_player.position, Vector3.zero, recenterThreshold))
            {
                Vector3 shift = CalculateShift(_player.position);
                ApplyShift(shift);
                _currentOriginOffset -= shift;
            }
        }

        public static bool ShouldRecenter(Vector3 playerPos, Vector3 origin, float threshold)
        {
            Vector3 offset = playerPos - origin;
            offset.y = 0f; // Only consider XZ distance
            return offset.magnitude > threshold;
        }

        public static Vector3 CalculateShift(Vector3 playerPos)
        {
            return new Vector3(-playerPos.x, 0f, -playerPos.z);
        }

        private void ApplyShift(Vector3 shift)
        {
            // Shift all root GameObjects in loaded scenes
            var rootObjects = new System.Collections.Generic.List<GameObject>();
            for (int i = 0; i < UnityEngine.SceneManagement.SceneManager.sceneCount; i++)
            {
                var scene = UnityEngine.SceneManagement.SceneManager.GetSceneAt(i);
                if (scene.isLoaded)
                {
                    scene.GetRootGameObjects(rootObjects);
                    foreach (var obj in rootObjects)
                    {
                        obj.transform.position += shift;
                    }
                    rootObjects.Clear();
                }
            }

            // Shift particle systems, audio sources are moved with their parent transforms
            Debug.Log($"[FloatingOrigin] Recentered by {shift}, total offset: {_currentOriginOffset}");
        }
    }
}
```

- [ ] **Step 3: Run tests**

Expected: All 3 PASS

- [ ] **Step 4: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Streaming/FloatingOrigin.cs Assets/Tests/EditMode/Streaming/FloatingOriginTests.cs
git commit -m "feat: implement floating origin with threshold-based recentering"
```

---

### Task 6: Boundary Stitcher

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Streaming/BoundaryStitcher.cs`

- [ ] **Step 1: Write BoundaryStitcher**

```csharp
// Assets/LonelyHighway/Scripts/Streaming/BoundaryStitcher.cs
using UnityEngine;
using System.Collections.Generic;

namespace LonelyHighway.Streaming
{
    /// <summary>
    /// Snaps vertices at tile boundaries to eliminate visual seams.
    /// Uses shared edge IDs from the pipeline to match boundary features
    /// between adjacent tiles.
    /// </summary>
    public class BoundaryStitcher : MonoBehaviour
    {
        [Tooltip("Maximum distance (meters) to snap boundary vertices")]
        public float snapThreshold = 0.5f;

        /// <summary>
        /// Stitch two adjacent tile meshes at their shared boundary.
        /// Call when both tiles are loaded.
        /// </summary>
        public void StitchTiles(Vector2Int tileA, Vector2Int tileB, float tileSize)
        {
            // Determine shared edge direction
            Vector2Int diff = tileB - tileA;
            if (Mathf.Abs(diff.x) + Mathf.Abs(diff.y) != 1)
                return; // Not adjacent

            bool horizontal = diff.x != 0; // shared edge is vertical (north-south)
            float edgePosition = horizontal
                ? Mathf.Max(tileA.x, tileB.x) * tileSize  // X position of shared edge
                : Mathf.Max(tileA.y, tileB.y) * tileSize;  // Z position of shared edge

            // Find mesh renderers near the shared edge in both tiles
            // and snap their boundary vertices to matching positions
            // Implementation depends on how pipeline marks boundary vertices
            // (via shared edge IDs in tile metadata)

            Debug.Log($"[Stitcher] Stitching tiles {tileA} <-> {tileB} at edge {edgePosition}");
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Streaming/BoundaryStitcher.cs
git commit -m "feat: add boundary stitcher stub for tile seam elimination"
```

---

### Task 7: World Streamer (Main Controller)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Streaming/WorldStreamer.cs`

- [ ] **Step 1: Write WorldStreamer**

```csharp
// Assets/LonelyHighway/Scripts/Streaming/WorldStreamer.cs
using UnityEngine;
using System.Collections.Generic;
using System.Linq;
using LonelyHighway.Data;

namespace LonelyHighway.Streaming
{
    [RequireComponent(typeof(TileLoader))]
    [RequireComponent(typeof(FloatingOrigin))]
    [RequireComponent(typeof(BoundaryStitcher))]
    public class WorldStreamer : MonoBehaviour
    {
        [Header("Configuration")]
        public float tileSize = 512f;

        [Header("References")]
        public Transform player;
        public TileMetadata[] tileDatabase;

        private TileLoader _loader;
        private FloatingOrigin _floatingOrigin;
        private BoundaryStitcher _stitcher;
        private Vector2Int _currentPlayerTile;
        private bool _initialized;

        // Ring radii: Active=1 (3x3), Buffer=2 (5x5), LOD=3 (7x7)
        private const int ActiveRadius = 1;
        private const int BufferRadius = 2;
        private const int LODRadius = 3;

        private void Awake()
        {
            _loader = GetComponent<TileLoader>();
            _floatingOrigin = GetComponent<FloatingOrigin>();
            _stitcher = GetComponent<BoundaryStitcher>();
        }

        private void Start()
        {
            if (player == null)
            {
                Debug.LogError("[WorldStreamer] No player transform assigned!");
                return;
            }

            _floatingOrigin.Initialize(player);
            _currentPlayerTile = TileGrid.WorldToTile(player.position, tileSize);

            // Initial load of all rings
            LoadRing(TileGrid.GetRingTiles(_currentPlayerTile, ActiveRadius), TileRing.Active);
            LoadRing(TileGrid.GetRingTiles(_currentPlayerTile, BufferRadius), TileRing.Buffer);
            LoadRing(TileGrid.GetRingTiles(_currentPlayerTile, LODRadius), TileRing.LOD);

            _loader.OnTileLoaded += HandleTileLoaded;
            _initialized = true;
        }

        private void Update()
        {
            if (!_initialized || player == null) return;

            Vector2Int newTile = TileGrid.WorldToTile(player.position, tileSize);
            if (newTile == _currentPlayerTile) return;

            // Player crossed a tile boundary — update rings
            var oldLODTiles = TileGrid.GetRingTiles(_currentPlayerTile, LODRadius);
            var newLODTiles = TileGrid.GetRingTiles(newTile, LODRadius);

            // Unload tiles that fell outside the LOD ring
            var toUnload = TileGrid.TileDifference(oldLODTiles, newLODTiles);
            foreach (var coord in toUnload)
                _loader.RequestUnload(coord);

            // Load new tiles that entered the LOD ring
            var toLoad = TileGrid.TileDifference(newLODTiles, oldLODTiles);
            foreach (var coord in toLoad)
            {
                var ring = TileState.DetermineRing(coord, newTile);
                var meta = FindTileMetadata(coord);
                if (meta != null)
                    _loader.RequestLoad(coord, ring, meta.sceneAddress);
            }

            // Update ring classifications for existing tiles
            foreach (var state in _loader.GetLoadedTiles())
            {
                var newRing = TileState.DetermineRing(state.Coord, newTile);
                if (newRing != state.Ring)
                    _loader.UpdateRing(state.Coord, newRing);
            }

            _currentPlayerTile = newTile;
        }

        private void HandleTileLoaded(Vector2Int coord, TileRing ring)
        {
            // Stitch with adjacent loaded tiles
            Vector2Int[] neighbors = {
                coord + Vector2Int.up, coord + Vector2Int.down,
                coord + Vector2Int.left, coord + Vector2Int.right
            };

            foreach (var neighbor in neighbors)
            {
                var neighborState = _loader.GetTileState(neighbor);
                if (neighborState != null && neighborState.LoadState == TileLoadState.Loaded)
                {
                    _stitcher.StitchTiles(coord, neighbor, tileSize);
                }
            }
        }

        private void LoadRing(Vector2Int[] tiles, TileRing ring)
        {
            foreach (var coord in tiles)
            {
                var meta = FindTileMetadata(coord);
                if (meta != null)
                    _loader.RequestLoad(coord, ring, meta.sceneAddress);
            }
        }

        private TileMetadata FindTileMetadata(Vector2Int coord)
        {
            if (tileDatabase == null) return null;
            foreach (var meta in tileDatabase)
            {
                if (meta.coordX == coord.x && meta.coordY == coord.y)
                    return meta;
            }
            return null;
        }

        private void OnDestroy()
        {
            if (_loader != null)
                _loader.OnTileLoaded -= HandleTileLoaded;
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Streaming/WorldStreamer.cs
git commit -m "feat: implement world streamer with ring-based tile management"
```

---

### Task 8: Pipeline Output Importer (Editor Script)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Streaming/TileImporter.cs`

- [ ] **Step 1: Write editor importer**

```csharp
// Assets/LonelyHighway/Scripts/Streaming/TileImporter.cs
#if UNITY_EDITOR
using UnityEngine;
using UnityEditor;
using System.IO;
using LonelyHighway.Data;

namespace LonelyHighway.Streaming
{
    /// <summary>
    /// Editor utility to import pipeline output (glb, json) into Unity assets.
    /// Menu: LonelyHighway > Import Pipeline Tiles
    /// </summary>
    public static class TileImporter
    {
        [MenuItem("LonelyHighway/Import Pipeline Tiles")]
        public static void ImportTiles()
        {
            string sourceDir = EditorUtility.OpenFolderPanel("Select Pipeline Output Directory", "", "");
            if (string.IsNullOrEmpty(sourceDir)) return;

            string targetDir = "Assets/GeneratedTiles";
            if (!AssetDatabase.IsValidFolder(targetDir))
                AssetDatabase.CreateFolder("Assets", "GeneratedTiles");

            int imported = 0;

            foreach (string metaPath in Directory.GetFiles(sourceDir, "*_meta.json"))
            {
                string baseName = Path.GetFileNameWithoutExtension(metaPath).Replace("_meta", "");

                // Read metadata
                string metaJson = File.ReadAllText(metaPath);
                var metaData = JsonUtility.FromJson<TileMetadataJson>(metaJson);

                // Create TileMetadata ScriptableObject
                var tileMeta = ScriptableObject.CreateInstance<TileMetadata>();
                tileMeta.coordX = metaData.coord_x;
                tileMeta.coordY = metaData.coord_y;
                tileMeta.minX = metaData.bounds.min_x;
                tileMeta.minZ = metaData.bounds.min_z;
                tileMeta.maxX = metaData.bounds.max_x;
                tileMeta.maxZ = metaData.bounds.max_z;
                tileMeta.roadMeshCount = metaData.road_count;
                tileMeta.buildingMeshCount = metaData.building_count;
                tileMeta.trafficNodeCount = metaData.traffic_nodes;
                tileMeta.trafficEdgeCount = metaData.traffic_edges;

                if (metaData.signal_positions != null)
                {
                    tileMeta.signalPositions = new Vector2[metaData.signal_positions.Length];
                    for (int i = 0; i < metaData.signal_positions.Length; i++)
                    {
                        tileMeta.signalPositions[i] = new Vector2(
                            metaData.signal_positions[i][0],
                            metaData.signal_positions[i][1]);
                    }
                }

                string assetPath = $"{targetDir}/{baseName}_meta.asset";
                AssetDatabase.CreateAsset(tileMeta, assetPath);

                // Copy glb file
                string glbSource = Path.Combine(sourceDir, baseName + ".glb");
                if (File.Exists(glbSource))
                {
                    string glbTarget = $"{targetDir}/{baseName}.glb";
                    FileUtil.ReplaceFile(glbSource, glbTarget);
                }

                // Copy minimap
                string minimapSource = Path.Combine(sourceDir, baseName + "_minimap.png");
                if (File.Exists(minimapSource))
                {
                    string minimapTarget = $"{targetDir}/{baseName}_minimap.png";
                    FileUtil.ReplaceFile(minimapSource, minimapTarget);
                }

                imported++;
            }

            AssetDatabase.Refresh();
            Debug.Log($"[TileImporter] Imported {imported} tiles to {targetDir}");
        }

        // JSON structure matching pipeline output
        [System.Serializable]
        private class TileMetadataJson
        {
            public int coord_x;
            public int coord_y;
            public TileBoundsJson bounds;
            public int road_count;
            public int building_count;
            public int traffic_nodes;
            public int traffic_edges;
            public float[][] signal_positions;
        }

        [System.Serializable]
        private class TileBoundsJson
        {
            public float min_x;
            public float min_z;
            public float max_x;
            public float max_z;
        }
    }
}
#endif
```

- [ ] **Step 2: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Streaming/TileImporter.cs
git commit -m "feat: add editor importer for pipeline tile output"
```

---

## Summary

| Task | Component | Tests |
|------|-----------|-------|
| 1 | Assembly defs + data types | — |
| 2 | Tile grid math | 9 unit tests |
| 3 | Tile state machine | — |
| 4 | Tile loader (Addressables) | — |
| 5 | Floating origin | 3 unit tests |
| 6 | Boundary stitcher (stub) | — |
| 7 | World streamer controller | — |
| 8 | Pipeline output importer | — |

**Total: 8 tasks, 12 unit tests**

## Deferred

| Feature | When |
|---------|------|
| Full boundary vertex stitching | After pipeline produces shared edge IDs |
| LOD impostor generation | After art pipeline provides impostor meshes |
| Traffic AI pool integration | Traffic AI plan (Plan 4) |
| Memory budget monitoring | Polish phase |
