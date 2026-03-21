# Traffic AI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the traffic simulation that populates Shenzhen with dense, reactive AI vehicles, buses, e-bikes, pedestrians, and traffic signals — all driven by the lane graph from the OSM pipeline.

**Architecture:** A `TrafficManager` spawns and pools AI vehicles. Each vehicle runs three behavior layers: Path (A* route), Drive (IDM car-following), React (dynamic events). A `SignalManager` runs signal phase cycles. Vehicles exist at three LOD levels matching the world streaming rings: full sim, rail, and ghost. A `PedestrianManager` handles crosswalk spawning.

**Tech Stack:** Unity 2022.3 LTS, C#, Unity Test Framework

**Spec reference:** `docs/superpowers/specs/2026-03-21-lonely-highway-design.md` — Section 5 (Traffic AI)

**Dependencies:** Lane graph data from OSM Pipeline (Plan 2), tile ring info from World Streaming (Plan 3)

---

## File Structure

```
Assets/
  LonelyHighway/
    Scripts/
      Traffic/
        TrafficManager.cs             — Top-level: spawning, pooling, LOD management
        AIVehicle.cs                  — MonoBehaviour on each AI car, runs behavior layers
        PathLayer.cs                  — A* pathfinding on lane graph
        DriveLayer.cs                 — IDM car-following, speed control, signal compliance
        ReactLayer.cs                 — Dynamic reactions: player cut-in, emergency stop, lane change
        SignalManager.cs              — Runs all signal controllers, provides phase queries
        LaneGraphRuntime.cs           — Runtime lane graph loaded from TrafficGraphData
        VehiclePool.cs                — Object pooling for AI vehicles
        DensityController.cs          — Time-of-day and weather density modifiers
        BusAI.cs                      — Bus-specific behavior: routes, stops, re-merge
        EBikeAI.cs                    — E-bike/scooter behavior: weaving, bike lanes
        PedestrianManager.cs          — Crosswalk pedestrian spawning and walk cycles
        Pedestrian.cs                 — Individual pedestrian behavior
      Data/
        AIVehicleProfile.cs           — ScriptableObject: AI vehicle tuning (accel, decel, size)
        TrafficConfig.cs              — ScriptableObject: pool sizes, density curves, spawn rules
  Tests/
    EditMode/
      Traffic/
        PathLayerTests.cs             — A* pathfinding tests
        DriveLayerTests.cs            — IDM model tests
        SignalManagerTests.cs         — Signal phase cycle tests
        DensityControllerTests.cs     — Density calculation tests
        LaneGraphRuntimeTests.cs      — Graph query tests
```

---

### Task 1: Assembly & Data Definitions

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/LonelyHighway.Traffic.asmdef`
- Create: `Assets/LonelyHighway/Scripts/Data/AIVehicleProfile.cs`
- Create: `Assets/LonelyHighway/Scripts/Data/TrafficConfig.cs`
- Create: `Assets/Tests/EditMode/Traffic/EditModeTrafficTests.asmdef`

- [ ] **Step 1: Create assembly definitions**

`LonelyHighway.Traffic.asmdef`:
```json
{
  "name": "LonelyHighway.Traffic",
  "rootNamespace": "LonelyHighway.Traffic",
  "references": ["LonelyHighway.Data", "LonelyHighway.Vehicle", "LonelyHighway.Streaming"],
  "includePlatforms": [],
  "autoReferenced": true
}
```

`EditModeTrafficTests.asmdef`:
```json
{
  "name": "EditModeTrafficTests",
  "rootNamespace": "LonelyHighway.Tests.EditMode.Traffic",
  "references": ["LonelyHighway.Traffic", "LonelyHighway.Data"],
  "includePlatforms": ["Editor"],
  "defineConstraints": ["UNITY_INCLUDE_TESTS"],
  "optionalUnityReferences": ["TestAssemblies"]
}
```

- [ ] **Step 2: Write AIVehicleProfile**

```csharp
// Assets/LonelyHighway/Scripts/Data/AIVehicleProfile.cs
using UnityEngine;

namespace LonelyHighway.Data
{
    [CreateAssetMenu(fileName = "NewAIVehicleProfile", menuName = "LonelyHighway/AI Vehicle Profile")]
    public class AIVehicleProfile : ScriptableObject
    {
        [Header("Identity")]
        public string vehicleName = "Sedan";
        public AIVehicleType vehicleType = AIVehicleType.Car;

        [Header("Dimensions")]
        public float length = 4.5f;
        public float width = 1.8f;

        [Header("Performance")]
        public float maxSpeed = 33f;        // m/s (~120 km/h)
        public float maxAcceleration = 3f;   // m/s^2
        public float comfortDecel = 2.5f;    // m/s^2 (IDM parameter)
        public float maxDeceleration = 8f;   // m/s^2 (emergency)

        [Header("IDM Parameters")]
        public float desiredTimeGap = 1.5f;  // seconds
        public float minimumGap = 2.0f;      // meters
        public float politeness = 0.5f;      // 0 = aggressive, 1 = polite

        [Header("Visuals")]
        public GameObject prefab;
    }

    public enum AIVehicleType
    {
        Car,
        Bus,
        EBike,
        Truck,
    }
}
```

- [ ] **Step 3: Write TrafficConfig**

```csharp
// Assets/LonelyHighway/Scripts/Data/TrafficConfig.cs
using UnityEngine;

namespace LonelyHighway.Data
{
    [CreateAssetMenu(fileName = "TrafficConfig", menuName = "LonelyHighway/Traffic Config")]
    public class TrafficConfig : ScriptableObject
    {
        [Header("Pool Sizes")]
        public int maxFullSimVehicles = 300;
        public int maxRailVehicles = 500;

        [Header("Density (vehicles per km of road)")]
        public float rushHourDensity = 40f;
        public float middayDensity = 20f;
        public float nightDensity = 8f;

        [Header("Time of Day (game hours 0-24)")]
        public float morningRushStart = 7f;
        public float morningRushEnd = 9f;
        public float eveningRushStart = 17f;
        public float eveningRushEnd = 19f;
        public float nightStart = 23f;
        public float nightEnd = 5f;

        [Header("Weather Multiplier")]
        public float heavyWeatherMultiplier = 0.8f;

        [Header("Special Vehicles")]
        [Range(0f, 0.3f)] public float busRatio = 0.05f;
        [Range(0f, 0.5f)] public float eBikeRatio = 0.15f;

        [Header("Pedestrians")]
        public int maxPedestrians = 50;
        public float pedestrianSpawnRadius = 100f;

        [Header("Spawn")]
        public float spawnDistance = 200f;
        public float despawnDistance = 300f;
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/ Assets/LonelyHighway/Scripts/Data/AIVehicleProfile.cs Assets/LonelyHighway/Scripts/Data/TrafficConfig.cs Assets/Tests/EditMode/Traffic/
git commit -m "feat: add traffic AI assembly, vehicle profiles, and traffic config"
```

---

### Task 2: Runtime Lane Graph

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/LaneGraphRuntime.cs`
- Create: `Assets/Tests/EditMode/Traffic/LaneGraphRuntimeTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Traffic/LaneGraphRuntimeTests.cs
using NUnit.Framework;
using LonelyHighway.Traffic;
using System.Collections.Generic;

namespace LonelyHighway.Tests.EditMode.Traffic
{
    public class LaneGraphRuntimeTests
    {
        private LaneGraphRuntime MakeSimpleGraph()
        {
            // Simple 3-node graph: 0 -> 1 -> 2
            var graph = new LaneGraphRuntime();
            graph.AddNode(0, 0f, 0f, 0f);
            graph.AddNode(1, 100f, 0f, 0f);
            graph.AddNode(2, 200f, 0f, 0f);
            graph.AddEdge(0, 0, 1, 60f, 0, 100f);
            graph.AddEdge(1, 1, 2, 60f, 0, 100f);
            return graph;
        }

        [Test]
        public void GetOutgoingEdges_ReturnsCorrectEdges()
        {
            var graph = MakeSimpleGraph();
            var edges = graph.GetOutgoingEdges(0);
            Assert.AreEqual(1, edges.Count);
            Assert.AreEqual(1, edges[0].toNode);
        }

        [Test]
        public void GetOutgoingEdges_LeafNode_ReturnsEmpty()
        {
            var graph = MakeSimpleGraph();
            var edges = graph.GetOutgoingEdges(2);
            Assert.AreEqual(0, edges.Count);
        }

        [Test]
        public void GetNodePosition_ReturnsCorrect()
        {
            var graph = MakeSimpleGraph();
            var pos = graph.GetNodePosition(1);
            Assert.AreEqual(100f, pos.x, 0.01f);
        }

        [Test]
        public void FindNearestNode_ReturnsClosest()
        {
            var graph = MakeSimpleGraph();
            int nearest = graph.FindNearestNode(90f, 5f);
            Assert.AreEqual(1, nearest);
        }

        [Test]
        public void EdgeCount_Correct()
        {
            var graph = MakeSimpleGraph();
            Assert.AreEqual(2, graph.EdgeCount);
        }
    }
}
```

- [ ] **Step 2: Write LaneGraphRuntime**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/LaneGraphRuntime.cs
using UnityEngine;
using System.Collections.Generic;
using LonelyHighway.Data;

namespace LonelyHighway.Traffic
{
    public struct RuntimeLaneNode
    {
        public int id;
        public float x, z, y;
    }

    public struct RuntimeLaneEdge
    {
        public int id;
        public int fromNode;
        public int toNode;
        public float speedLimitKmh;
        public int laneIndex;
        public float length;
    }

    public class LaneGraphRuntime
    {
        private readonly List<RuntimeLaneNode> _nodes = new();
        private readonly List<RuntimeLaneEdge> _edges = new();
        private readonly Dictionary<int, List<RuntimeLaneEdge>> _outgoing = new();

        public int NodeCount => _nodes.Count;
        public int EdgeCount => _edges.Count;

        public void AddNode(int id, float x, float z, float y)
        {
            _nodes.Add(new RuntimeLaneNode { id = id, x = x, z = z, y = y });
            if (!_outgoing.ContainsKey(id))
                _outgoing[id] = new List<RuntimeLaneEdge>();
        }

        public void AddEdge(int id, int from, int to, float speedLimit, int laneIndex, float length)
        {
            var edge = new RuntimeLaneEdge
            {
                id = id, fromNode = from, toNode = to,
                speedLimitKmh = speedLimit, laneIndex = laneIndex, length = length
            };
            _edges.Add(edge);

            if (!_outgoing.ContainsKey(from))
                _outgoing[from] = new List<RuntimeLaneEdge>();
            _outgoing[from].Add(edge);
        }

        public List<RuntimeLaneEdge> GetOutgoingEdges(int nodeId)
        {
            return _outgoing.TryGetValue(nodeId, out var edges) ? edges : new List<RuntimeLaneEdge>();
        }

        public Vector3 GetNodePosition(int nodeId)
        {
            var node = _nodes[nodeId];
            return new Vector3(node.x, node.y, node.z);
        }

        public int FindNearestNode(float x, float z)
        {
            int nearest = -1;
            float bestDist = float.MaxValue;
            for (int i = 0; i < _nodes.Count; i++)
            {
                float dx = _nodes[i].x - x;
                float dz = _nodes[i].z - z;
                float dist = dx * dx + dz * dz;
                if (dist < bestDist)
                {
                    bestDist = dist;
                    nearest = i;
                }
            }
            return nearest;
        }

        /// <summary>
        /// Load from TrafficGraphData ScriptableObject.
        /// </summary>
        public void LoadFromData(TrafficGraphData data)
        {
            foreach (var n in data.nodes)
                AddNode(n.id, n.x, n.z, n.y);
            foreach (var e in data.edges)
                AddEdge(e.id, e.fromNode, e.toNode, e.speedLimitKmh, e.laneIndex, e.length);
        }
    }
}
```

- [ ] **Step 3: Run tests**

Expected: All 5 PASS

- [ ] **Step 4: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/LaneGraphRuntime.cs Assets/Tests/EditMode/Traffic/LaneGraphRuntimeTests.cs
git commit -m "feat: implement runtime lane graph with queries and nearest-node search"
```

---

### Task 3: A* Pathfinding (Path Layer)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/PathLayer.cs`
- Create: `Assets/Tests/EditMode/Traffic/PathLayerTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Traffic/PathLayerTests.cs
using NUnit.Framework;
using LonelyHighway.Traffic;
using System.Collections.Generic;

namespace LonelyHighway.Tests.EditMode.Traffic
{
    public class PathLayerTests
    {
        private LaneGraphRuntime MakeGraph()
        {
            // 0 -> 1 -> 2 -> 3 (straight)
            //      1 -> 4 -> 3 (detour, longer)
            var g = new LaneGraphRuntime();
            g.AddNode(0, 0f, 0f, 0f);
            g.AddNode(1, 100f, 0f, 0f);
            g.AddNode(2, 200f, 0f, 0f);
            g.AddNode(3, 300f, 0f, 0f);
            g.AddNode(4, 150f, 100f, 0f);

            g.AddEdge(0, 0, 1, 60f, 0, 100f);
            g.AddEdge(1, 1, 2, 60f, 0, 100f);
            g.AddEdge(2, 2, 3, 60f, 0, 100f);
            g.AddEdge(3, 1, 4, 60f, 0, 112f);
            g.AddEdge(4, 4, 3, 60f, 0, 180f);
            return g;
        }

        [Test]
        public void FindPath_DirectRoute_ReturnsShortest()
        {
            var graph = MakeGraph();
            var path = PathLayer.FindPath(graph, 0, 3);
            Assert.IsNotNull(path);
            Assert.AreEqual(4, path.Count); // 0, 1, 2, 3
            Assert.AreEqual(0, path[0]);
            Assert.AreEqual(3, path[3]);
        }

        [Test]
        public void FindPath_NoRoute_ReturnsNull()
        {
            var graph = MakeGraph();
            var path = PathLayer.FindPath(graph, 3, 0); // No backward edges
            Assert.IsNull(path);
        }

        [Test]
        public void FindPath_SameNode_ReturnsSingle()
        {
            var graph = MakeGraph();
            var path = PathLayer.FindPath(graph, 2, 2);
            Assert.IsNotNull(path);
            Assert.AreEqual(1, path.Count);
        }

        [Test]
        public void FindPath_ChoosesShorterRoute()
        {
            var graph = MakeGraph();
            var path = PathLayer.FindPath(graph, 1, 3);
            // Direct: 1->2->3 = 200m, Detour: 1->4->3 = 292m
            Assert.AreEqual(3, path.Count); // 1, 2, 3
            Assert.AreEqual(2, path[1]); // Goes through 2, not 4
        }
    }
}
```

- [ ] **Step 2: Write A* implementation**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/PathLayer.cs
using System.Collections.Generic;
using UnityEngine;

namespace LonelyHighway.Traffic
{
    public static class PathLayer
    {
        /// <summary>
        /// A* pathfinding on the lane graph. Returns list of node IDs, or null if no path.
        /// </summary>
        public static List<int> FindPath(LaneGraphRuntime graph, int startNode, int goalNode)
        {
            if (startNode == goalNode)
                return new List<int> { startNode };

            var openSet = new SortedSet<(float fScore, int node)>();
            var gScore = new Dictionary<int, float>();
            var cameFrom = new Dictionary<int, int>();
            var inOpen = new HashSet<int>();

            gScore[startNode] = 0f;
            var goalPos = graph.GetNodePosition(goalNode);
            float h = Heuristic(graph.GetNodePosition(startNode), goalPos);
            openSet.Add((h, startNode));
            inOpen.Add(startNode);

            while (openSet.Count > 0)
            {
                var (_, current) = openSet.Min;
                openSet.Remove(openSet.Min);
                inOpen.Remove(current);

                if (current == goalNode)
                    return ReconstructPath(cameFrom, current);

                foreach (var edge in graph.GetOutgoingEdges(current))
                {
                    float tentative = gScore[current] + edge.length;
                    if (!gScore.ContainsKey(edge.toNode) || tentative < gScore[edge.toNode])
                    {
                        cameFrom[edge.toNode] = current;
                        gScore[edge.toNode] = tentative;
                        float f = tentative + Heuristic(graph.GetNodePosition(edge.toNode), goalPos);

                        if (!inOpen.Contains(edge.toNode))
                        {
                            openSet.Add((f, edge.toNode));
                            inOpen.Add(edge.toNode);
                        }
                    }
                }
            }

            return null; // No path found
        }

        private static float Heuristic(Vector3 a, Vector3 b)
        {
            float dx = a.x - b.x;
            float dz = a.z - b.z;
            return Mathf.Sqrt(dx * dx + dz * dz);
        }

        private static List<int> ReconstructPath(Dictionary<int, int> cameFrom, int current)
        {
            var path = new List<int> { current };
            while (cameFrom.ContainsKey(current))
            {
                current = cameFrom[current];
                path.Insert(0, current);
            }
            return path;
        }
    }
}
```

- [ ] **Step 3: Run tests**

Expected: All 4 PASS

- [ ] **Step 4: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/PathLayer.cs Assets/Tests/EditMode/Traffic/PathLayerTests.cs
git commit -m "feat: implement A* pathfinding on lane graph"
```

---

### Task 4: IDM Car-Following (Drive Layer)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/DriveLayer.cs`
- Create: `Assets/Tests/EditMode/Traffic/DriveLayerTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Traffic/DriveLayerTests.cs
using NUnit.Framework;
using LonelyHighway.Traffic;

namespace LonelyHighway.Tests.EditMode.Traffic
{
    public class DriveLayerTests
    {
        [Test]
        public void IDM_FreeRoad_AcceleratesToDesiredSpeed()
        {
            float accel = DriveLayer.CalculateIDMAcceleration(
                speed: 10f, desiredSpeed: 30f,
                gap: 500f, deltaV: 0f,
                maxAccel: 3f, comfortDecel: 2.5f,
                desiredTimeGap: 1.5f, minimumGap: 2f);
            Assert.Greater(accel, 0f, "Should accelerate on free road");
        }

        [Test]
        public void IDM_AtDesiredSpeed_ZeroAcceleration()
        {
            float accel = DriveLayer.CalculateIDMAcceleration(
                speed: 30f, desiredSpeed: 30f,
                gap: 500f, deltaV: 0f,
                maxAccel: 3f, comfortDecel: 2.5f,
                desiredTimeGap: 1.5f, minimumGap: 2f);
            Assert.AreEqual(0f, accel, 0.3f);
        }

        [Test]
        public void IDM_CloseToLeader_Decelerates()
        {
            float accel = DriveLayer.CalculateIDMAcceleration(
                speed: 20f, desiredSpeed: 30f,
                gap: 5f, deltaV: 0f,
                maxAccel: 3f, comfortDecel: 2.5f,
                desiredTimeGap: 1.5f, minimumGap: 2f);
            Assert.Less(accel, 0f, "Should decelerate when close to leader");
        }

        [Test]
        public void IDM_ApproachingFaster_StrongDeceleration()
        {
            float accel = DriveLayer.CalculateIDMAcceleration(
                speed: 25f, desiredSpeed: 30f,
                gap: 15f, deltaV: 10f, // approaching 10 m/s faster
                maxAccel: 3f, comfortDecel: 2.5f,
                desiredTimeGap: 1.5f, minimumGap: 2f);
            Assert.Less(accel, -1f, "Should brake hard when approaching fast");
        }

        [Test]
        public void IDM_Stopped_WithGap_Accelerates()
        {
            float accel = DriveLayer.CalculateIDMAcceleration(
                speed: 0f, desiredSpeed: 30f,
                gap: 50f, deltaV: 0f,
                maxAccel: 3f, comfortDecel: 2.5f,
                desiredTimeGap: 1.5f, minimumGap: 2f);
            Assert.Greater(accel, 0f);
        }

        [Test]
        public void ShouldStopForSignal_RedLight_ReturnsTrue()
        {
            bool stop = DriveLayer.ShouldStopForSignal(
                distanceToSignal: 30f, speed: 15f, isGreen: false);
            Assert.IsTrue(stop);
        }

        [Test]
        public void ShouldStopForSignal_GreenLight_ReturnsFalse()
        {
            bool stop = DriveLayer.ShouldStopForSignal(
                distanceToSignal: 30f, speed: 15f, isGreen: true);
            Assert.IsFalse(stop);
        }

        [Test]
        public void ShouldStopForSignal_RedButTooClose_ReturnsFalse()
        {
            // Can't stop safely — already in intersection
            bool stop = DriveLayer.ShouldStopForSignal(
                distanceToSignal: 3f, speed: 15f, isGreen: false);
            Assert.IsFalse(stop);
        }
    }
}
```

- [ ] **Step 2: Write IDM implementation**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/DriveLayer.cs
using UnityEngine;

namespace LonelyHighway.Traffic
{
    public static class DriveLayer
    {
        /// <summary>
        /// Intelligent Driver Model (IDM) acceleration calculation.
        /// </summary>
        /// <param name="speed">Current vehicle speed (m/s)</param>
        /// <param name="desiredSpeed">Desired speed / speed limit (m/s)</param>
        /// <param name="gap">Distance to leading vehicle (m)</param>
        /// <param name="deltaV">Speed difference with leader: self - leader (m/s). Positive = approaching</param>
        /// <param name="maxAccel">Maximum acceleration (m/s^2)</param>
        /// <param name="comfortDecel">Comfortable deceleration (m/s^2)</param>
        /// <param name="desiredTimeGap">Desired time gap (seconds)</param>
        /// <param name="minimumGap">Minimum gap at standstill (meters)</param>
        /// <returns>Acceleration in m/s^2 (positive = accel, negative = decel)</returns>
        public static float CalculateIDMAcceleration(
            float speed, float desiredSpeed,
            float gap, float deltaV,
            float maxAccel, float comfortDecel,
            float desiredTimeGap, float minimumGap)
        {
            // Free-road acceleration term
            float speedRatio = desiredSpeed > 0f ? speed / desiredSpeed : 1f;
            float freeAccel = maxAccel * (1f - Mathf.Pow(speedRatio, 4));

            // Desired dynamic gap
            float sStar = minimumGap
                + Mathf.Max(0f, speed * desiredTimeGap
                    + (speed * deltaV) / (2f * Mathf.Sqrt(maxAccel * comfortDecel)));

            // Interaction term
            float interactionDecel = gap > 0.01f
                ? maxAccel * (sStar / gap) * (sStar / gap)
                : maxAccel; // Avoid division by zero

            return freeAccel - interactionDecel;
        }

        /// <summary>
        /// Determine if the vehicle should stop for a traffic signal.
        /// </summary>
        /// <param name="distanceToSignal">Distance to stop line (meters)</param>
        /// <param name="speed">Current speed (m/s)</param>
        /// <param name="isGreen">Whether the signal is green</param>
        /// <returns>True if vehicle should decelerate to stop</returns>
        public static bool ShouldStopForSignal(float distanceToSignal, float speed, bool isGreen)
        {
            if (isGreen) return false;

            // If too close to stop safely (within ~1 second at current speed), don't brake
            float minStopDistance = speed * 0.5f; // rough comfort stop distance
            if (distanceToSignal < minStopDistance && distanceToSignal < 5f)
                return false;

            return true;
        }

        /// <summary>
        /// Calculate curve deceleration factor based on road curvature.
        /// </summary>
        public static float CurveSpeedFactor(float curvatureRadius)
        {
            if (curvatureRadius <= 0f) return 1f;
            // Comfortable cornering: v = sqrt(a_lateral * r), limit lateral accel to 3 m/s^2
            float maxCurveSpeed = Mathf.Sqrt(3f * curvatureRadius);
            return maxCurveSpeed;
        }
    }
}
```

- [ ] **Step 3: Run tests**

Expected: All 8 PASS

- [ ] **Step 4: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/DriveLayer.cs Assets/Tests/EditMode/Traffic/DriveLayerTests.cs
git commit -m "feat: implement IDM car-following model and signal compliance"
```

---

### Task 5: Signal Manager

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/SignalManager.cs`
- Create: `Assets/Tests/EditMode/Traffic/SignalManagerTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Traffic/SignalManagerTests.cs
using NUnit.Framework;
using LonelyHighway.Traffic;

namespace LonelyHighway.Tests.EditMode.Traffic
{
    public class SignalManagerTests
    {
        [Test]
        public void NewSignal_StartsGreen()
        {
            var signal = new SignalState(90f, new[] {
                (SignalPhase.Green, 40f),
                (SignalPhase.Yellow, 3f),
                (SignalPhase.Red, 47f)
            });
            Assert.AreEqual(SignalPhase.Green, signal.CurrentPhase);
        }

        [Test]
        public void AfterGreenDuration_TransitionsToYellow()
        {
            var signal = new SignalState(90f, new[] {
                (SignalPhase.Green, 40f),
                (SignalPhase.Yellow, 3f),
                (SignalPhase.Red, 47f)
            });
            signal.Update(41f); // Past green duration
            Assert.AreEqual(SignalPhase.Yellow, signal.CurrentPhase);
        }

        [Test]
        public void FullCycle_WrapsToGreen()
        {
            var signal = new SignalState(90f, new[] {
                (SignalPhase.Green, 40f),
                (SignalPhase.Yellow, 3f),
                (SignalPhase.Red, 47f)
            });
            signal.Update(91f); // Past full cycle
            Assert.AreEqual(SignalPhase.Green, signal.CurrentPhase);
        }

        [Test]
        public void IsGreen_DuringGreen_ReturnsTrue()
        {
            var signal = new SignalState(90f, new[] {
                (SignalPhase.Green, 40f),
                (SignalPhase.Yellow, 3f),
                (SignalPhase.Red, 47f)
            });
            Assert.IsTrue(signal.IsGreen);
        }

        [Test]
        public void IsGreen_DuringRed_ReturnsFalse()
        {
            var signal = new SignalState(90f, new[] {
                (SignalPhase.Green, 40f),
                (SignalPhase.Yellow, 3f),
                (SignalPhase.Red, 47f)
            });
            signal.Update(50f);
            Assert.IsFalse(signal.IsGreen);
        }
    }
}
```

- [ ] **Step 2: Write SignalManager**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/SignalManager.cs
using UnityEngine;
using System.Collections.Generic;

namespace LonelyHighway.Traffic
{
    public enum SignalPhase
    {
        Green,
        Yellow,
        Red,
        LeftTurnArrow,
        PedestrianWalk,
    }

    public class SignalState
    {
        public float CycleTime { get; }
        public SignalPhase CurrentPhase { get; private set; }
        public bool IsGreen => CurrentPhase == SignalPhase.Green || CurrentPhase == SignalPhase.LeftTurnArrow;

        private readonly (SignalPhase phase, float duration)[] _phases;
        private float _elapsed;

        public SignalState(float cycleTime, (SignalPhase, float)[] phases)
        {
            CycleTime = cycleTime;
            _phases = phases;
            _elapsed = 0f;
            CurrentPhase = _phases.Length > 0 ? _phases[0].phase : SignalPhase.Red;
        }

        public void Update(float deltaTime)
        {
            _elapsed += deltaTime;
            float cyclePos = _elapsed % CycleTime;

            float accumulated = 0f;
            foreach (var (phase, duration) in _phases)
            {
                accumulated += duration;
                if (cyclePos < accumulated)
                {
                    CurrentPhase = phase;
                    return;
                }
            }
            CurrentPhase = _phases[_phases.Length - 1].phase;
        }
    }

    public class SignalManager : MonoBehaviour
    {
        private readonly Dictionary<long, SignalState> _signals = new();

        public void RegisterSignal(long id, float cycleTime, (SignalPhase, float)[] phases)
        {
            _signals[id] = new SignalState(cycleTime, phases);
        }

        private void FixedUpdate()
        {
            float dt = Time.fixedDeltaTime;
            foreach (var signal in _signals.Values)
                signal.Update(dt);
        }

        public SignalState GetSignal(long id)
        {
            return _signals.TryGetValue(id, out var state) ? state : null;
        }

        public bool IsGreen(long signalId)
        {
            var signal = GetSignal(signalId);
            return signal?.IsGreen ?? true; // Default to green if no signal
        }
    }
}
```

- [ ] **Step 3: Run tests**

Expected: All 5 PASS

- [ ] **Step 4: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/SignalManager.cs Assets/Tests/EditMode/Traffic/SignalManagerTests.cs
git commit -m "feat: implement signal phase controller with cycle management"
```

---

### Task 6: React Layer & AI Vehicle

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/ReactLayer.cs`
- Create: `Assets/LonelyHighway/Scripts/Traffic/AIVehicle.cs`

- [ ] **Step 1: Write ReactLayer**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/ReactLayer.cs
using UnityEngine;

namespace LonelyHighway.Traffic
{
    public static class ReactLayer
    {
        /// <summary>
        /// Check if the player vehicle has cut in front of this AI vehicle.
        /// </summary>
        public static bool DetectCutIn(
            Vector3 aiPosition, Vector3 aiForward, float aiSpeed,
            Vector3 playerPosition, Vector3 playerVelocity,
            float detectionRange, float laneWidth)
        {
            Vector3 toPlayer = playerPosition - aiPosition;
            float forwardDot = Vector3.Dot(toPlayer, aiForward);
            float lateralDist = Mathf.Abs(Vector3.Dot(toPlayer, Vector3.Cross(aiForward, Vector3.up)));

            // Player is ahead, within lane width, and within detection range
            return forwardDot > 0f && forwardDot < detectionRange
                && lateralDist < laneWidth
                && Vector3.Dot(playerVelocity.normalized, aiForward) > 0.5f; // Moving roughly same direction
        }

        /// <summary>
        /// Calculate emergency braking deceleration.
        /// </summary>
        public static float EmergencyBrakeAcceleration(float speed, float gap, float maxDecel)
        {
            if (gap <= 0f) return -maxDecel;
            // Time to collision
            float ttc = gap / Mathf.Max(speed, 0.1f);
            if (ttc < 1.5f) return -maxDecel;
            if (ttc < 3f) return -maxDecel * 0.5f;
            return 0f;
        }

        /// <summary>
        /// Decide whether to change lanes to pass a slow vehicle.
        /// Uses MOBIL lane-change model (simplified).
        /// </summary>
        public static bool ShouldChangeLane(
            float currentAccel, float afterChangeAccel,
            float followerAccelBefore, float followerAccelAfter,
            float politeness, float threshold)
        {
            // MOBIL: change if own advantage outweighs disadvantage to follower
            float ownAdvantage = afterChangeAccel - currentAccel;
            float followerDisadvantage = politeness * (followerAccelBefore - followerAccelAfter);
            return ownAdvantage - followerDisadvantage > threshold;
        }
    }
}
```

- [ ] **Step 2: Write AIVehicle MonoBehaviour**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/AIVehicle.cs
using UnityEngine;
using System.Collections.Generic;
using LonelyHighway.Data;

namespace LonelyHighway.Traffic
{
    public class AIVehicle : MonoBehaviour
    {
        public AIVehicleProfile profile;

        // Current state
        public float Speed { get; private set; }
        public int CurrentEdgeIndex { get; private set; }
        public float EdgeProgress { get; private set; } // 0-1 along current edge
        public bool IsActive { get; set; }

        private List<int> _path;
        private int _pathIndex;
        private LaneGraphRuntime _graph;
        private SignalManager _signals;
        private Transform _playerTransform;

        public void Initialize(
            LaneGraphRuntime graph, SignalManager signals,
            List<int> path, Transform player, AIVehicleProfile profile)
        {
            _graph = graph;
            _signals = signals;
            _path = path;
            _pathIndex = 0;
            _playerTransform = player;
            this.profile = profile;
            Speed = 0f;
            EdgeProgress = 0f;
            IsActive = true;

            if (_path != null && _path.Count > 0)
                transform.position = _graph.GetNodePosition(_path[0]);
        }

        public void SimulateStep(float dt)
        {
            if (!IsActive || _path == null || _pathIndex >= _path.Count - 1)
                return;

            int fromNode = _path[_pathIndex];
            int toNode = _path[_pathIndex + 1];
            var edges = _graph.GetOutgoingEdges(fromNode);
            RuntimeLaneEdge? currentEdge = null;
            foreach (var e in edges)
            {
                if (e.toNode == toNode) { currentEdge = e; break; }
            }
            if (!currentEdge.HasValue) return;

            float edgeLength = currentEdge.Value.length;
            float speedLimit = currentEdge.Value.speedLimitKmh / 3.6f;

            // IDM acceleration (simplified: use large gap if no leader detected)
            float gap = 100f; // TODO: scan for leader on same edge
            float deltaV = 0f;

            // Check for player cut-in
            if (_playerTransform != null)
            {
                float playerDist = Vector3.Distance(transform.position, _playerTransform.position);
                if (playerDist < gap && ReactLayer.DetectCutIn(
                    transform.position, transform.forward, Speed,
                    _playerTransform.position, Vector3.zero, 50f, 4f))
                {
                    gap = playerDist;
                }
            }

            float accel = DriveLayer.CalculateIDMAcceleration(
                Speed, speedLimit, gap, deltaV,
                profile.maxAcceleration, profile.comfortDecel,
                profile.desiredTimeGap, profile.minimumGap);

            // Emergency brake check
            float emergencyAccel = ReactLayer.EmergencyBrakeAcceleration(Speed, gap, profile.maxDeceleration);
            if (emergencyAccel < accel)
                accel = emergencyAccel;

            Speed = Mathf.Max(0f, Speed + accel * dt);
            EdgeProgress += (Speed * dt) / Mathf.Max(edgeLength, 0.1f);

            // Advance to next edge
            if (EdgeProgress >= 1f)
            {
                EdgeProgress = 0f;
                _pathIndex++;
                if (_pathIndex < _path.Count)
                    transform.position = _graph.GetNodePosition(_path[_pathIndex]);
            }
            else
            {
                // Interpolate position along edge
                Vector3 fromPos = _graph.GetNodePosition(fromNode);
                Vector3 toPos = _graph.GetNodePosition(toNode);
                transform.position = Vector3.Lerp(fromPos, toPos, EdgeProgress);

                Vector3 dir = (toPos - fromPos).normalized;
                if (dir.sqrMagnitude > 0.001f)
                    transform.forward = dir;
            }
        }

        /// <summary>
        /// Simplified rail sim — follow path at constant speed, no AI decisions.
        /// </summary>
        public void SimulateRail(float dt, float railSpeed)
        {
            if (_path == null || _pathIndex >= _path.Count - 1) return;

            int fromNode = _path[_pathIndex];
            int toNode = _path[_pathIndex + 1];
            var edges = _graph.GetOutgoingEdges(fromNode);
            float edgeLength = 100f;
            foreach (var e in edges)
            {
                if (e.toNode == toNode) { edgeLength = e.length; break; }
            }

            Speed = railSpeed;
            EdgeProgress += (railSpeed * dt) / Mathf.Max(edgeLength, 0.1f);

            if (EdgeProgress >= 1f)
            {
                EdgeProgress = 0f;
                _pathIndex++;
            }

            if (_pathIndex < _path.Count - 1)
            {
                Vector3 fromPos = _graph.GetNodePosition(_path[_pathIndex]);
                Vector3 toPos = _graph.GetNodePosition(_path[_pathIndex + 1]);
                transform.position = Vector3.Lerp(fromPos, toPos, EdgeProgress);
                Vector3 dir = (toPos - fromPos).normalized;
                if (dir.sqrMagnitude > 0.001f) transform.forward = dir;
            }
        }
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/ReactLayer.cs Assets/LonelyHighway/Scripts/Traffic/AIVehicle.cs
git commit -m "feat: implement react layer (cut-in, emergency brake, lane change) and AI vehicle"
```

---

### Task 7: Vehicle Pool & Density Controller

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/VehiclePool.cs`
- Create: `Assets/LonelyHighway/Scripts/Traffic/DensityController.cs`
- Create: `Assets/Tests/EditMode/Traffic/DensityControllerTests.cs`

- [ ] **Step 1: Write DensityController tests**

```csharp
// Assets/Tests/EditMode/Traffic/DensityControllerTests.cs
using NUnit.Framework;
using LonelyHighway.Traffic;
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode.Traffic
{
    public class DensityControllerTests
    {
        private TrafficConfig MakeConfig()
        {
            var config = ScriptableObject.CreateInstance<TrafficConfig>();
            config.rushHourDensity = 40f;
            config.middayDensity = 20f;
            config.nightDensity = 8f;
            config.morningRushStart = 7f;
            config.morningRushEnd = 9f;
            config.eveningRushStart = 17f;
            config.eveningRushEnd = 19f;
            config.nightStart = 23f;
            config.nightEnd = 5f;
            config.heavyWeatherMultiplier = 0.8f;
            return config;
        }

        [Test]
        public void RushHour_ReturnsMaxDensity()
        {
            var config = MakeConfig();
            float density = DensityController.CalculateDensity(8f, false, config);
            Assert.AreEqual(40f, density, 0.1f);
            Object.DestroyImmediate(config);
        }

        [Test]
        public void Midday_ReturnsModerateDensity()
        {
            var config = MakeConfig();
            float density = DensityController.CalculateDensity(12f, false, config);
            Assert.AreEqual(20f, density, 0.1f);
            Object.DestroyImmediate(config);
        }

        [Test]
        public void Night_ReturnsSparse()
        {
            var config = MakeConfig();
            float density = DensityController.CalculateDensity(2f, false, config);
            Assert.AreEqual(8f, density, 0.1f);
            Object.DestroyImmediate(config);
        }

        [Test]
        public void HeavyWeather_ReducesDensity()
        {
            var config = MakeConfig();
            float normal = DensityController.CalculateDensity(12f, false, config);
            float rainy = DensityController.CalculateDensity(12f, true, config);
            Assert.Less(rainy, normal);
            Assert.AreEqual(normal * 0.8f, rainy, 0.1f);
            Object.DestroyImmediate(config);
        }
    }
}
```

- [ ] **Step 2: Write DensityController**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/DensityController.cs
using LonelyHighway.Data;

namespace LonelyHighway.Traffic
{
    public static class DensityController
    {
        /// <summary>
        /// Calculate traffic density (vehicles per km) for a given time and weather.
        /// </summary>
        /// <param name="gameHour">Current game time (0-24)</param>
        /// <param name="isHeavyWeather">Rain, fog, or storm active</param>
        /// <param name="config">Traffic configuration</param>
        public static float CalculateDensity(float gameHour, bool isHeavyWeather, TrafficConfig config)
        {
            float baseDensity;

            if (IsInRange(gameHour, config.morningRushStart, config.morningRushEnd) ||
                IsInRange(gameHour, config.eveningRushStart, config.eveningRushEnd))
            {
                baseDensity = config.rushHourDensity;
            }
            else if (IsInRange(gameHour, config.nightStart, config.nightEnd))
            {
                baseDensity = config.nightDensity;
            }
            else
            {
                baseDensity = config.middayDensity;
            }

            if (isHeavyWeather)
                baseDensity *= config.heavyWeatherMultiplier;

            return baseDensity;
        }

        private static bool IsInRange(float hour, float start, float end)
        {
            if (start <= end)
                return hour >= start && hour < end;
            // Wraps midnight (e.g., 23-5)
            return hour >= start || hour < end;
        }
    }
}
```

- [ ] **Step 3: Write VehiclePool**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/VehiclePool.cs
using UnityEngine;
using System.Collections.Generic;

namespace LonelyHighway.Traffic
{
    public class VehiclePool : MonoBehaviour
    {
        public GameObject defaultVehiclePrefab;
        public int initialPoolSize = 100;

        private readonly Queue<AIVehicle> _available = new();
        private readonly HashSet<AIVehicle> _active = new();

        public int ActiveCount => _active.Count;
        public int AvailableCount => _available.Count;

        private void Start()
        {
            for (int i = 0; i < initialPoolSize; i++)
                CreatePooledVehicle();
        }

        public AIVehicle Acquire()
        {
            AIVehicle vehicle;
            if (_available.Count > 0)
            {
                vehicle = _available.Dequeue();
            }
            else
            {
                vehicle = CreatePooledVehicle();
                _available.Dequeue(); // Remove it from available since we're returning it
            }

            vehicle.gameObject.SetActive(true);
            _active.Add(vehicle);
            return vehicle;
        }

        public void Release(AIVehicle vehicle)
        {
            vehicle.IsActive = false;
            vehicle.gameObject.SetActive(false);
            _active.Remove(vehicle);
            _available.Enqueue(vehicle);
        }

        public void ReleaseAll()
        {
            foreach (var v in new List<AIVehicle>(_active))
                Release(v);
        }

        private AIVehicle CreatePooledVehicle()
        {
            var go = Instantiate(defaultVehiclePrefab, transform);
            go.SetActive(false);
            var vehicle = go.GetComponent<AIVehicle>();
            if (vehicle == null) vehicle = go.AddComponent<AIVehicle>();
            _available.Enqueue(vehicle);
            return vehicle;
        }
    }
}
```

- [ ] **Step 4: Run tests**

Expected: All 4 PASS

- [ ] **Step 5: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/VehiclePool.cs Assets/LonelyHighway/Scripts/Traffic/DensityController.cs Assets/Tests/EditMode/Traffic/DensityControllerTests.cs
git commit -m "feat: implement vehicle pool and time-of-day density controller"
```

---

### Task 8: Special Vehicles (Bus & E-Bike)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/BusAI.cs`
- Create: `Assets/LonelyHighway/Scripts/Traffic/EBikeAI.cs`

- [ ] **Step 1: Write BusAI**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/BusAI.cs
using UnityEngine;

namespace LonelyHighway.Traffic
{
    /// <summary>
    /// Bus-specific behavior layered on top of AIVehicle.
    /// Buses follow designated bus lanes, stop at bus stops, and re-merge.
    /// </summary>
    public class BusAI : MonoBehaviour
    {
        public float stopDuration = 20f; // seconds at each stop

        private AIVehicle _vehicle;
        private float _stopTimer;
        private bool _isAtStop;

        private void Awake()
        {
            _vehicle = GetComponent<AIVehicle>();
        }

        /// <summary>
        /// Check if the bus should pull over at the current position.
        /// Called each sim step by TrafficManager.
        /// </summary>
        public bool ShouldStop(Vector3 busStopPosition, float arrivalThreshold)
        {
            float dist = Vector3.Distance(transform.position, busStopPosition);
            return dist < arrivalThreshold && !_isAtStop;
        }

        public void BeginStop()
        {
            _isAtStop = true;
            _stopTimer = stopDuration + Random.Range(-5f, 10f); // 15-30s
        }

        /// <summary>
        /// Returns true while the bus is stopped. Decrement timer.
        /// </summary>
        public bool UpdateStop(float dt)
        {
            if (!_isAtStop) return false;

            _stopTimer -= dt;
            if (_stopTimer <= 0f)
            {
                _isAtStop = false;
                return false;
            }
            return true;
        }
    }
}
```

- [ ] **Step 2: Write EBikeAI**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/EBikeAI.cs
using UnityEngine;

namespace LonelyHighway.Traffic
{
    /// <summary>
    /// E-bike/scooter behavior layered on top of AIVehicle.
    /// E-bikes are smaller, slower, and weave between lanes.
    /// </summary>
    public class EBikeAI : MonoBehaviour
    {
        [Header("Weaving")]
        public float weaveAmplitude = 1.5f;  // meters lateral offset
        public float weaveFrequency = 0.3f;  // cycles per second
        public float maxSpeed = 12f;         // m/s (~43 km/h)

        private AIVehicle _vehicle;
        private float _weavePhase;

        private void Awake()
        {
            _vehicle = GetComponent<AIVehicle>();
            _weavePhase = Random.Range(0f, Mathf.PI * 2f);
        }

        /// <summary>
        /// Calculate lateral offset for weaving behavior.
        /// Apply this to the vehicle's position perpendicular to travel direction.
        /// </summary>
        public float GetLateralOffset(float time)
        {
            return Mathf.Sin(time * weaveFrequency * Mathf.PI * 2f + _weavePhase) * weaveAmplitude;
        }

        /// <summary>
        /// Clamp the AI vehicle's speed to e-bike maximum.
        /// </summary>
        public float ClampSpeed(float requestedSpeed)
        {
            return Mathf.Min(requestedSpeed, maxSpeed);
        }
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/BusAI.cs Assets/LonelyHighway/Scripts/Traffic/EBikeAI.cs
git commit -m "feat: implement bus stop behavior and e-bike weaving"
```

---

### Task 9: Pedestrians

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/PedestrianManager.cs`
- Create: `Assets/LonelyHighway/Scripts/Traffic/Pedestrian.cs`

- [ ] **Step 1: Write Pedestrian**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/Pedestrian.cs
using UnityEngine;

namespace LonelyHighway.Traffic
{
    public class Pedestrian : MonoBehaviour
    {
        public float walkSpeed = 1.4f; // m/s average walking speed

        private Vector3 _startPoint;
        private Vector3 _endPoint;
        private float _progress; // 0-1
        private bool _isWalking;
        private bool _isWaiting;

        public bool IsDone => _progress >= 1f;

        public void SetCrossing(Vector3 start, Vector3 end)
        {
            _startPoint = start;
            _endPoint = end;
            _progress = 0f;
            _isWaiting = true;
            _isWalking = false;
            transform.position = start;
        }

        public void StartWalking()
        {
            _isWaiting = false;
            _isWalking = true;
        }

        public void UpdatePedestrian(float dt)
        {
            if (!_isWalking || IsDone) return;

            float distance = Vector3.Distance(_startPoint, _endPoint);
            if (distance < 0.1f) { _progress = 1f; return; }

            _progress += (walkSpeed * dt) / distance;
            _progress = Mathf.Clamp01(_progress);

            transform.position = Vector3.Lerp(_startPoint, _endPoint, _progress);
            Vector3 dir = (_endPoint - _startPoint).normalized;
            if (dir.sqrMagnitude > 0.001f)
                transform.forward = dir;
        }
    }
}
```

- [ ] **Step 2: Write PedestrianManager**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/PedestrianManager.cs
using UnityEngine;
using System.Collections.Generic;
using LonelyHighway.Data;

namespace LonelyHighway.Traffic
{
    public class PedestrianManager : MonoBehaviour
    {
        public GameObject pedestrianPrefab;
        public TrafficConfig config;
        public SignalManager signalManager;

        private readonly List<Pedestrian> _active = new();
        private readonly Queue<Pedestrian> _pool = new();

        /// <summary>
        /// Spawn a group of pedestrians at a crosswalk when the walk signal is active.
        /// </summary>
        public void SpawnGroup(Vector3 start, Vector3 end, int count)
        {
            for (int i = 0; i < count; i++)
            {
                var ped = AcquirePedestrian();
                // Stagger positions slightly
                Vector3 offset = new Vector3(
                    Random.Range(-1f, 1f),
                    0f,
                    Random.Range(-0.5f, 0.5f));
                ped.SetCrossing(start + offset, end + offset);
                ped.StartWalking();
            }
        }

        private void FixedUpdate()
        {
            float dt = Time.fixedDeltaTime;

            for (int i = _active.Count - 1; i >= 0; i--)
            {
                _active[i].UpdatePedestrian(dt);

                if (_active[i].IsDone)
                {
                    ReleasePedestrian(_active[i]);
                    _active.RemoveAt(i);
                }
            }
        }

        private Pedestrian AcquirePedestrian()
        {
            Pedestrian ped;
            if (_pool.Count > 0)
            {
                ped = _pool.Dequeue();
                ped.gameObject.SetActive(true);
            }
            else
            {
                var go = Instantiate(pedestrianPrefab, transform);
                ped = go.GetComponent<Pedestrian>();
                if (ped == null) ped = go.AddComponent<Pedestrian>();
            }
            _active.Add(ped);
            return ped;
        }

        private void ReleasePedestrian(Pedestrian ped)
        {
            ped.gameObject.SetActive(false);
            _pool.Enqueue(ped);
        }
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/Pedestrian.cs Assets/LonelyHighway/Scripts/Traffic/PedestrianManager.cs
git commit -m "feat: implement pedestrian crosswalk system with pooling"
```

---

### Task 10: Traffic Manager (Main Controller)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Traffic/TrafficManager.cs`

- [ ] **Step 1: Write TrafficManager**

```csharp
// Assets/LonelyHighway/Scripts/Traffic/TrafficManager.cs
using UnityEngine;
using System.Collections.Generic;
using LonelyHighway.Data;
using LonelyHighway.Streaming;

namespace LonelyHighway.Traffic
{
    [RequireComponent(typeof(VehiclePool))]
    [RequireComponent(typeof(SignalManager))]
    [RequireComponent(typeof(PedestrianManager))]
    public class TrafficManager : MonoBehaviour
    {
        [Header("Configuration")]
        public TrafficConfig config;
        public AIVehicleProfile[] vehicleProfiles;

        [Header("References")]
        public Transform player;
        public TileLoader tileLoader;

        private VehiclePool _pool;
        private SignalManager _signals;
        private PedestrianManager _pedestrians;
        private LaneGraphRuntime _activeGraph;
        private readonly List<AIVehicle> _fullSimVehicles = new();
        private readonly List<AIVehicle> _railVehicles = new();

        public float GameHour { get; set; } // Set by Environment system
        public bool IsHeavyWeather { get; set; } // Set by Environment system

        private void Awake()
        {
            _pool = GetComponent<VehiclePool>();
            _signals = GetComponent<SignalManager>();
            _pedestrians = GetComponent<PedestrianManager>();
            _activeGraph = new LaneGraphRuntime();
        }

        /// <summary>
        /// Called by WorldStreamer when a tile is loaded.
        /// Merges the tile's traffic graph into the active graph and spawns vehicles.
        /// </summary>
        public void OnTileLoaded(Vector2Int coord, TileRing ring, TrafficGraphData graphData)
        {
            if (graphData != null)
                _activeGraph.LoadFromData(graphData);

            if (ring == TileRing.Active)
                SpawnVehiclesForTile(coord);
        }

        /// <summary>
        /// Called by WorldStreamer when a tile is unloaded.
        /// Despawns vehicles on that tile and removes its graph data.
        /// </summary>
        public void OnTileUnloaded(Vector2Int coord)
        {
            // Release vehicles that were on this tile
            for (int i = _fullSimVehicles.Count - 1; i >= 0; i--)
            {
                var v = _fullSimVehicles[i];
                var vTile = TileGrid.WorldToTile(v.transform.position, 512f);
                if (vTile == coord)
                {
                    _pool.Release(v);
                    _fullSimVehicles.RemoveAt(i);
                }
            }
        }

        private void FixedUpdate()
        {
            float dt = Time.fixedDeltaTime;

            // Full sim vehicles — complete AI
            foreach (var v in _fullSimVehicles)
                v.SimulateStep(dt);

            // Rail vehicles — simplified movement
            foreach (var v in _railVehicles)
                v.SimulateRail(dt, 15f); // ~54 km/h default rail speed
        }

        private void SpawnVehiclesForTile(Vector2Int coord)
        {
            float density = DensityController.CalculateDensity(GameHour, IsHeavyWeather, config);
            int targetCount = Mathf.RoundToInt(density * 0.512f); // density per km * tile size in km

            if (_pool.ActiveCount >= config.maxFullSimVehicles)
                return;

            for (int i = 0; i < targetCount && _pool.ActiveCount < config.maxFullSimVehicles; i++)
            {
                var vehicle = _pool.Acquire();
                var profile = vehicleProfiles[Random.Range(0, vehicleProfiles.Length)];

                // Pick random start and end nodes
                if (_activeGraph.NodeCount < 2) continue;
                int startNode = Random.Range(0, _activeGraph.NodeCount);
                int endNode = Random.Range(0, _activeGraph.NodeCount);
                if (startNode == endNode) continue;

                var path = PathLayer.FindPath(_activeGraph, startNode, endNode);
                if (path == null || path.Count < 2)
                {
                    _pool.Release(vehicle);
                    continue;
                }

                vehicle.Initialize(_activeGraph, _signals, path, player, profile);
                _fullSimVehicles.Add(vehicle);
            }
        }

        /// <summary>
        /// Update ring classification for a vehicle — switch between full sim and rail.
        /// Called when WorldStreamer updates tile rings.
        /// </summary>
        public void UpdateVehicleRing(AIVehicle vehicle, TileRing newRing)
        {
            if (newRing == TileRing.Active && !_fullSimVehicles.Contains(vehicle))
            {
                _railVehicles.Remove(vehicle);
                _fullSimVehicles.Add(vehicle);
            }
            else if (newRing == TileRing.Buffer && !_railVehicles.Contains(vehicle))
            {
                _fullSimVehicles.Remove(vehicle);
                _railVehicles.Add(vehicle);
            }
            else if (newRing == TileRing.None || newRing == TileRing.LOD)
            {
                _fullSimVehicles.Remove(vehicle);
                _railVehicles.Remove(vehicle);
                _pool.Release(vehicle);
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Traffic/TrafficManager.cs
git commit -m "feat: implement traffic manager with spawning, LOD, and tile integration"
```

---

## Summary

| Task | Component | Tests |
|------|-----------|-------|
| 1 | Assembly defs + data types | — |
| 2 | Runtime lane graph | 5 unit tests |
| 3 | A* pathfinding | 4 unit tests |
| 4 | IDM car-following | 8 unit tests |
| 5 | Signal manager | 5 unit tests |
| 6 | React layer + AI vehicle | — |
| 7 | Vehicle pool + density | 4 unit tests |
| 8 | Bus + e-bike behaviors | — |
| 9 | Pedestrian system | — |
| 10 | Traffic manager | — |

**Total: 10 tasks, 26 unit tests**

## Deferred

| Feature | When |
|---------|------|
| Full MOBIL lane-change implementation | After basic traffic is working |
| Bus route matching from OSM `route=bus` | After pipeline supports route relations |
| Leader scanning on same edge (IDM gap) | Integration phase — needs spatial queries |
| Ghost sim (headlight particles) | Environment plan |
| Roundabout-specific AI behavior | After intersections are modeled in pipeline |
