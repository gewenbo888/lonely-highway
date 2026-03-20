# Vehicle Physics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a custom raycast vehicle controller with Pacejka tire model, realistic suspension, drivetrain, and damage system — playable on a test track before any city data exists.

**Architecture:** Custom raycast vehicle (no WheelCollider). Each wheel casts a ray downward to detect ground, applies spring-damper suspension force, then feeds slip values into a Pacejka tire model to compute grip. Drivetrain applies engine torque through gears and differential. Damage system tracks per-panel health and modifies physics parameters. All configurable via ScriptableObject vehicle profiles.

**Tech Stack:** Unity 2022.3 LTS, URP, C#, Unity Input System, Unity Test Framework (EditMode + PlayMode)

**Spec reference:** `docs/superpowers/specs/2026-03-21-lonely-highway-design.md` — Section 4 (Vehicle Physics)

---

## File Structure

```
Assets/
  LonelyHighway/
    Scripts/
      Vehicle/
        VehicleController.cs          — Top-level vehicle MonoBehaviour, owns all subsystems
        WheelRaycast.cs               — Per-wheel raycast ground detection
        SuspensionSystem.cs           — Spring-damper force calculation per wheel
        PacejkaTireModel.cs           — Pacejka Magic Formula: slip → force
        Drivetrain.cs                 — Engine torque curve, gears, differential
        SteeringSystem.cs             — Speed-sensitive steering with self-aligning torque
        WeightTransfer.cs             — Dynamic load distribution across wheels
        DamageSystem.cs               — Per-panel health, mechanical effects, recovery
        VehicleInput.cs               — Reads Input System actions, exposes normalized values
        VehicleAudio.cs               — Engine sound, tire screech, impact sounds (placeholder)
        SurfaceIdentifier.cs          — MonoBehaviour on ground colliders to identify surface type
        GarageInteraction.cs          — Trigger zone for garage full-repair (stub, wired to world data later)
      Data/
        VehicleProfile.cs             — ScriptableObject: all vehicle tuning parameters
        TireProfile.cs                — ScriptableObject: Pacejka coefficients per surface
        SurfaceType.cs                — Enum + surface-to-tire-profile mapping
      Camera/
        VehicleCameraController.cs    — Camera mode switching (interior, hood, chase, free)
        InteriorCamera.cs             — Dashboard cam with head bob/sway
        ChaseCamera.cs                — Follow camera with damping
        MirrorRenderer.cs             — Renders rear-view mirrors at half res, 30fps
    Data/
      Vehicles/
        BYD-Qin.asset                — VehicleProfile ScriptableObject for the starter sedan
      Tires/
        DryAsphalt.asset             — TireProfile for dry road
        WetAsphalt.asset             — TireProfile for wet road
        PaintedLine.asset            — TireProfile for road markings (slippery when wet)
    Scenes/
      TestTrack.unity                — Flat test scene with varied surfaces for physics tuning
    Prefabs/
      PlayerVehicle.prefab           — Assembled vehicle prefab

Assets/
  Tests/
    EditMode/
      Vehicle/
        PacejkaTireModelTests.cs     — Unit tests for tire force calculations
        SuspensionSystemTests.cs     — Unit tests for spring-damper forces
        DrivetrainTests.cs           — Unit tests for torque/gear calculations
        WeightTransferTests.cs       — Unit tests for load distribution
        DamageSystemTests.cs         — Unit tests for damage state and recovery
        SteeringSystemTests.cs       — Unit tests for steering ratio and self-align
    PlayMode/
      Vehicle/
        VehicleIntegrationTests.cs   — Integration tests: vehicle on ground, drives, steers
        SurfaceGripTests.cs          — Tests surface type affects grip correctly
```

---

### Task 1: Unity Project Setup

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/` (directory)
- Create: `Assets/LonelyHighway/Scripts/Data/` (directory)
- Create: `Assets/Tests/EditMode/Vehicle/` (directory)
- Create: `Assets/Tests/PlayMode/Vehicle/` (directory)
- Create: `Assets/LonelyHighway/Scenes/TestTrack.unity`

- [ ] **Step 1: Create Unity project**

Create a new Unity 2022.3 LTS project with URP template:
```bash
# If Unity Hub CLI is available:
unity-hub -- --createProject /Users/geir/Game/unity-project --template com.unity.template.urp
# Otherwise: create via Unity Hub GUI, project name "LonelyHighway", template "3D (URP)", location /Users/geir/Game/
```

- [ ] **Step 2: Install required packages**

In Unity Package Manager, install:
- `com.unity.inputsystem` (Input System)
- `com.unity.test-framework` (should be pre-installed)
- `com.unity.addressables` (needed later for streaming, install now)

Or edit `Packages/manifest.json` to add:
```json
{
  "com.unity.inputsystem": "1.7.0",
  "com.unity.addressables": "1.21.19"
}
```

- [ ] **Step 3: Enable Input System backend**

Edit > Project Settings > Player > Other Settings > Active Input Handling → set to "Both" (allows gradual migration).

- [ ] **Step 4: Create directory structure**

In Unity Editor Project window, create folders:
```
Assets/LonelyHighway/Scripts/Vehicle/
Assets/LonelyHighway/Scripts/Data/
Assets/LonelyHighway/Scripts/Camera/
Assets/LonelyHighway/Data/Vehicles/
Assets/LonelyHighway/Data/Tires/
Assets/LonelyHighway/Scenes/
Assets/LonelyHighway/Prefabs/
Assets/Tests/EditMode/Vehicle/
Assets/Tests/PlayMode/Vehicle/
```

- [ ] **Step 5: Create test assemblies**

Create `Assets/Tests/EditMode/Vehicle/EditModeVehicleTests.asmdef`:
```json
{
  "name": "EditModeVehicleTests",
  "rootNamespace": "LonelyHighway.Tests.EditMode",
  "references": ["LonelyHighway.Vehicle"],
  "includePlatforms": ["Editor"],
  "defineConstraints": ["UNITY_INCLUDE_TESTS"],
  "optionalUnityReferences": ["TestAssemblies"]
}
```

Create `Assets/Tests/PlayMode/Vehicle/PlayModeVehicleTests.asmdef`:
```json
{
  "name": "PlayModeVehicleTests",
  "rootNamespace": "LonelyHighway.Tests.PlayMode",
  "references": ["LonelyHighway.Vehicle"],
  "includePlatforms": [],
  "defineConstraints": ["UNITY_INCLUDE_TESTS"],
  "optionalUnityReferences": ["TestAssemblies"]
}
```

Create `Assets/LonelyHighway/Scripts/Vehicle/LonelyHighway.Vehicle.asmdef`:
```json
{
  "name": "LonelyHighway.Vehicle",
  "rootNamespace": "LonelyHighway.Vehicle",
  "references": ["Unity.InputSystem"],
  "includePlatforms": [],
  "autoReferenced": true
}
```

Create `Assets/LonelyHighway/Scripts/Data/LonelyHighway.Data.asmdef`:
```json
{
  "name": "LonelyHighway.Data",
  "rootNamespace": "LonelyHighway.Data",
  "references": [],
  "includePlatforms": [],
  "autoReferenced": true
}
```

Update `LonelyHighway.Vehicle.asmdef` to reference Data:
```json
{
  "name": "LonelyHighway.Vehicle",
  "rootNamespace": "LonelyHighway.Vehicle",
  "references": ["Unity.InputSystem", "LonelyHighway.Data"],
  "includePlatforms": [],
  "autoReferenced": true
}
```

- [ ] **Step 6: Create test track scene**

Create a new scene `Assets/LonelyHighway/Scenes/TestTrack.unity`:
- A large flat plane (200m x 200m) with a default material tagged as "DryAsphalt"
- A directional light
- A smaller plane section (20m x 20m) with "WetAsphalt" material for surface transition testing
- A painted line strip (2m wide, 50m long) with "PaintedLine" material
- Save scene

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: Unity project setup with URP, Input System, test assemblies, test track"
```

---

### Task 2: Data Definitions (ScriptableObjects)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Data/SurfaceType.cs`
- Create: `Assets/LonelyHighway/Scripts/Data/TireProfile.cs`
- Create: `Assets/LonelyHighway/Scripts/Data/VehicleProfile.cs`

- [ ] **Step 1: Write SurfaceType enum**

```csharp
// Assets/LonelyHighway/Scripts/Data/SurfaceType.cs
namespace LonelyHighway.Data
{
    public enum SurfaceType
    {
        DryAsphalt,
        WetAsphalt,
        Concrete,
        PaintedLine,
        Gravel,
        Grass
    }
}
```

- [ ] **Step 2: Write TireProfile ScriptableObject**

```csharp
// Assets/LonelyHighway/Scripts/Data/TireProfile.cs
using UnityEngine;

namespace LonelyHighway.Data
{
    [CreateAssetMenu(fileName = "NewTireProfile", menuName = "LonelyHighway/Tire Profile")]
    public class TireProfile : ScriptableObject
    {
        [Header("Pacejka Lateral (Fy) Coefficients")]
        [Tooltip("Peak factor")] public float lateralB = 10f;
        [Tooltip("Shape factor")] public float lateralC = 1.9f;
        [Tooltip("Peak value")] public float lateralD = 1.0f;
        [Tooltip("Curvature")] public float lateralE = -0.97f;

        [Header("Pacejka Longitudinal (Fx) Coefficients")]
        [Tooltip("Peak factor")] public float longitudinalB = 12f;
        [Tooltip("Shape factor")] public float longitudinalC = 2.3f;
        [Tooltip("Peak value")] public float longitudinalD = 1.0f;
        [Tooltip("Curvature")] public float longitudinalE = -0.96f;
    }
}
```

- [ ] **Step 3: Write VehicleProfile ScriptableObject**

```csharp
// Assets/LonelyHighway/Scripts/Data/VehicleProfile.cs
using UnityEngine;

namespace LonelyHighway.Data
{
    [CreateAssetMenu(fileName = "NewVehicleProfile", menuName = "LonelyHighway/Vehicle Profile")]
    public class VehicleProfile : ScriptableObject
    {
        [Header("Dimensions")]
        public float mass = 1500f;
        public Vector3 centerOfMass = new Vector3(0f, 0.3f, 0.2f);

        [Header("Suspension")]
        public float springRate = 35000f;
        public float damperRate = 4500f;
        public float restLength = 0.35f;
        public float maxTravel = 0.15f;
        public float antiRollBarStiffness = 5000f;

        [Header("Wheels")]
        public float wheelRadius = 0.33f;
        public float wheelMass = 15f;
        [Tooltip("Front axle distance from center of mass")]
        public float frontAxleOffset = 1.35f;
        [Tooltip("Rear axle distance from center of mass")]
        public float rearAxleOffset = 1.4f;
        [Tooltip("Half-width between left and right wheels")]
        public float trackHalfWidth = 0.78f;

        [Header("Engine")]
        public AnimationCurve torqueCurve = AnimationCurve.Linear(0f, 0f, 7000f, 250f);
        public float maxRPM = 7000f;
        public float idleRPM = 800f;
        public float engineBraking = 50f;

        [Header("Transmission")]
        public float[] gearRatios = { 3.6f, 2.1f, 1.4f, 1.0f, 0.77f, 0.63f };
        public float reverseGearRatio = 3.2f;
        public float finalDriveRatio = 3.7f;
        public float shiftUpRPM = 6500f;
        public float shiftDownRPM = 2500f;

        [Header("Steering")]
        public float maxSteerAngle = 35f;
        public float steerSpeedFactor = 0.5f;
        [Tooltip("Minimum steering ratio at high speed (multiplied by maxSteerAngle)")]
        public float highSpeedSteerMultiplier = 0.3f;
        [Tooltip("Speed (m/s) at which steering reaches minimum ratio")]
        public float steerLimitSpeed = 30f;

        [Header("Brakes")]
        public float maxBrakeForce = 5000f;
        public float handbrakeForce = 3000f;
        [Tooltip("Front brake bias (0-1). 0.6 = 60% front")]
        public float brakeBias = 0.6f;

        [Header("Damage")]
        public float collisionDamageThreshold = 3f;
        public float alignmentDriftRate = 0.5f;
        public float engineStutterChance = 0.1f;
        public float passiveRecoveryRate = 0.05f;
        public float suspensionSagRate = 0.3f;
    }
}
```

- [ ] **Step 4: Verify scripts compile**

Open Unity Editor — ensure no compile errors in Console.

- [ ] **Step 5: Create asset instances**

In Unity Editor:
- Right-click `Assets/LonelyHighway/Data/Vehicles/` → Create → LonelyHighway → Vehicle Profile → name it "BYD-Qin"
- Right-click `Assets/LonelyHighway/Data/Tires/` → Create → LonelyHighway → Tire Profile → name it "DryAsphalt", set default values (already in code)
- Duplicate for "WetAsphalt" — reduce D values to 0.7 (lateralD = 0.7, longitudinalD = 0.7)
- Duplicate for "PaintedLine" — reduce D values to 0.5

- [ ] **Step 6: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Data/ Assets/LonelyHighway/Data/
git commit -m "feat: add VehicleProfile, TireProfile, SurfaceType data definitions"
```

---

### Task 3: Pacejka Tire Model

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/PacejkaTireModel.cs`
- Create: `Assets/Tests/EditMode/Vehicle/PacejkaTireModelTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Vehicle/PacejkaTireModelTests.cs
using NUnit.Framework;
using LonelyHighway.Vehicle;
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode
{
    public class PacejkaTireModelTests
    {
        private TireProfile _profile;

        [SetUp]
        public void SetUp()
        {
            _profile = ScriptableObject.CreateInstance<TireProfile>();
            // Default values from TireProfile: B=10, C=1.9, D=1.0, E=-0.97
        }

        [TearDown]
        public void TearDown()
        {
            Object.DestroyImmediate(_profile);
        }

        [Test]
        public void LateralForce_AtZeroSlip_ReturnsZero()
        {
            float force = PacejkaTireModel.CalculateLateralForce(0f, 5000f, _profile);
            Assert.AreEqual(0f, force, 0.01f);
        }

        [Test]
        public void LateralForce_AtSmallSlip_ReturnsProportionalForce()
        {
            float force = PacejkaTireModel.CalculateLateralForce(0.05f, 5000f, _profile);
            // Should be positive and significant but not at peak
            Assert.Greater(force, 0f);
            Assert.Less(force, 5000f);
        }

        [Test]
        public void LateralForce_ScalesWithNormalLoad()
        {
            float forceLight = PacejkaTireModel.CalculateLateralForce(0.1f, 3000f, _profile);
            float forceHeavy = PacejkaTireModel.CalculateLateralForce(0.1f, 6000f, _profile);
            Assert.Greater(forceHeavy, forceLight);
        }

        [Test]
        public void LongitudinalForce_AtZeroSlip_ReturnsZero()
        {
            float force = PacejkaTireModel.CalculateLongitudinalForce(0f, 5000f, _profile);
            Assert.AreEqual(0f, force, 0.01f);
        }

        [Test]
        public void LongitudinalForce_AtSmallSlip_ReturnsProportionalForce()
        {
            float force = PacejkaTireModel.CalculateLongitudinalForce(0.05f, 5000f, _profile);
            Assert.Greater(force, 0f);
            Assert.Less(force, 5000f);
        }

        [Test]
        public void LateralForce_NegativeSlip_ReturnsNegativeForce()
        {
            float force = PacejkaTireModel.CalculateLateralForce(-0.1f, 5000f, _profile);
            Assert.Less(force, 0f);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: Unity Editor → Window → General → Test Runner → EditMode → Run All
Expected: 6 failures — `PacejkaTireModel` class does not exist

- [ ] **Step 3: Write Pacejka implementation**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/PacejkaTireModel.cs
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public static class PacejkaTireModel
    {
        /// <summary>
        /// Pacejka Magic Formula: F = D * sin(C * atan(B*x - E*(B*x - atan(B*x))))
        /// where x = slip angle/ratio, scaled by normal load.
        /// </summary>
        private static float MagicFormula(float slip, float normalLoad, float B, float C, float D, float E)
        {
            float x = slip;
            float Bx = B * x;
            float force = normalLoad * D * Mathf.Sin(C * Mathf.Atan(Bx - E * (Bx - Mathf.Atan(Bx))));
            return force;
        }

        /// <summary>
        /// Calculate lateral (cornering) force from slip angle.
        /// </summary>
        /// <param name="slipAngle">Slip angle in radians</param>
        /// <param name="normalLoad">Normal force on tire in Newtons</param>
        /// <param name="profile">Tire profile with Pacejka coefficients</param>
        /// <returns>Lateral force in Newtons (positive = toward center of turn)</returns>
        public static float CalculateLateralForce(float slipAngle, float normalLoad, TireProfile profile)
        {
            return MagicFormula(slipAngle, normalLoad,
                profile.lateralB, profile.lateralC, profile.lateralD, profile.lateralE);
        }

        /// <summary>
        /// Calculate longitudinal (traction/braking) force from slip ratio.
        /// </summary>
        /// <param name="slipRatio">Slip ratio (-1 to 1). Positive = traction, negative = braking</param>
        /// <param name="normalLoad">Normal force on tire in Newtons</param>
        /// <param name="profile">Tire profile with Pacejka coefficients</param>
        /// <returns>Longitudinal force in Newtons</returns>
        public static float CalculateLongitudinalForce(float slipRatio, float normalLoad, TireProfile profile)
        {
            return MagicFormula(slipRatio, normalLoad,
                profile.longitudinalB, profile.longitudinalC, profile.longitudinalD, profile.longitudinalE);
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: Unity Editor → Test Runner → EditMode → Run All
Expected: 6 PASS

- [ ] **Step 5: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/PacejkaTireModel.cs Assets/Tests/EditMode/Vehicle/PacejkaTireModelTests.cs
git commit -m "feat: implement Pacejka tire model with Magic Formula"
```

---

### Task 4: Suspension System

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/SuspensionSystem.cs`
- Create: `Assets/Tests/EditMode/Vehicle/SuspensionSystemTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Vehicle/SuspensionSystemTests.cs
using NUnit.Framework;
using LonelyHighway.Vehicle;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode
{
    public class SuspensionSystemTests
    {
        [Test]
        public void SpringForce_AtRestLength_ReturnsZero()
        {
            float force = SuspensionSystem.CalculateSpringForce(
                compression: 0f, velocity: 0f,
                springRate: 35000f, damperRate: 4500f);
            Assert.AreEqual(0f, force, 0.01f);
        }

        [Test]
        public void SpringForce_Compressed_ReturnsPositiveForce()
        {
            float force = SuspensionSystem.CalculateSpringForce(
                compression: 0.05f, velocity: 0f,
                springRate: 35000f, damperRate: 4500f);
            Assert.Greater(force, 0f);
            // F = kx = 35000 * 0.05 = 1750
            Assert.AreEqual(1750f, force, 0.01f);
        }

        [Test]
        public void DamperForce_CompressingVelocity_AddsForce()
        {
            float forceStatic = SuspensionSystem.CalculateSpringForce(
                compression: 0.05f, velocity: 0f,
                springRate: 35000f, damperRate: 4500f);
            float forceDynamic = SuspensionSystem.CalculateSpringForce(
                compression: 0.05f, velocity: -0.5f, // compressing
                springRate: 35000f, damperRate: 4500f);
            Assert.Greater(forceDynamic, forceStatic);
        }

        [Test]
        public void SpringForce_FullyExtended_ReturnsZero()
        {
            // No ground contact — no force
            float force = SuspensionSystem.CalculateSpringForce(
                compression: -0.1f, velocity: 0f,
                springRate: 35000f, damperRate: 4500f);
            Assert.AreEqual(0f, force, 0.01f);
        }

        [Test]
        public void AntiRollForce_EqualCompression_ReturnsZero()
        {
            float force = SuspensionSystem.CalculateAntiRollForce(
                leftCompression: 0.05f, rightCompression: 0.05f,
                stiffness: 5000f);
            Assert.AreEqual(0f, force, 0.01f);
        }

        [Test]
        public void AntiRollForce_UnequalCompression_ReturnsCorrectiveForce()
        {
            float force = SuspensionSystem.CalculateAntiRollForce(
                leftCompression: 0.08f, rightCompression: 0.02f,
                stiffness: 5000f);
            // Should push compressed side up (positive) and extended side down
            Assert.Greater(force, 0f);
            // F = stiffness * (left - right) = 5000 * 0.06 = 300
            Assert.AreEqual(300f, force, 0.01f);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: Test Runner → EditMode → Run All
Expected: 6 failures — `SuspensionSystem` does not exist

- [ ] **Step 3: Write implementation**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/SuspensionSystem.cs
namespace LonelyHighway.Vehicle
{
    public static class SuspensionSystem
    {
        /// <summary>
        /// Calculate combined spring + damper force for one wheel.
        /// </summary>
        /// <param name="compression">How much the spring is compressed (meters). 0 = rest, positive = compressed, negative = extended beyond rest</param>
        /// <param name="velocity">Compression velocity (m/s). Negative = compressing, positive = extending</param>
        /// <param name="springRate">Spring stiffness in N/m</param>
        /// <param name="damperRate">Damper coefficient in Ns/m</param>
        /// <returns>Upward force in Newtons. Returns 0 if spring is extended beyond rest (no ground contact)</returns>
        public static float CalculateSpringForce(float compression, float velocity, float springRate, float damperRate)
        {
            if (compression <= 0f)
                return 0f;

            float spring = springRate * compression;
            float damper = damperRate * -velocity; // negative velocity = compressing = positive damper force
            float total = spring + damper;

            // Suspension can push but not pull
            return total > 0f ? total : 0f;
        }

        /// <summary>
        /// Calculate anti-roll bar force for one side of an axle.
        /// </summary>
        /// <param name="leftCompression">Left wheel compression in meters</param>
        /// <param name="rightCompression">Right wheel compression in meters</param>
        /// <param name="stiffness">Anti-roll bar stiffness in N/m</param>
        /// <returns>Force to apply to the left wheel (negate for right wheel)</returns>
        public static float CalculateAntiRollForce(float leftCompression, float rightCompression, float stiffness)
        {
            return stiffness * (leftCompression - rightCompression);
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: Test Runner → EditMode → Run All
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/SuspensionSystem.cs Assets/Tests/EditMode/Vehicle/SuspensionSystemTests.cs
git commit -m "feat: implement suspension spring-damper and anti-roll bar"
```

---

### Task 5: Drivetrain

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/Drivetrain.cs`
- Create: `Assets/Tests/EditMode/Vehicle/DrivetrainTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Vehicle/DrivetrainTests.cs
using NUnit.Framework;
using LonelyHighway.Vehicle;
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode
{
    public class DrivetrainTests
    {
        private VehicleProfile _profile;

        [SetUp]
        public void SetUp()
        {
            _profile = ScriptableObject.CreateInstance<VehicleProfile>();
            _profile.gearRatios = new float[] { 3.6f, 2.1f, 1.4f, 1.0f, 0.77f, 0.63f };
            _profile.reverseGearRatio = 3.2f;
            _profile.finalDriveRatio = 3.7f;
            _profile.maxRPM = 7000f;
            _profile.idleRPM = 800f;
            _profile.torqueCurve = AnimationCurve.Linear(0f, 0f, 7000f, 250f);
            _profile.shiftUpRPM = 6500f;
            _profile.shiftDownRPM = 2500f;
        }

        [TearDown]
        public void TearDown()
        {
            Object.DestroyImmediate(_profile);
        }

        [Test]
        public void WheelTorque_FirstGear_FullThrottle_ReturnsHighTorque()
        {
            var state = new DrivetrainState { currentGear = 0, rpm = 4000f };
            float torque = Drivetrain.CalculateWheelTorque(1f, state, _profile);
            // torque_at_4000 = lerp(0, 250, 4000/7000) ≈ 142.86
            // wheel_torque = 142.86 * 3.6 * 3.7 ≈ 1903.7
            Assert.Greater(torque, 1500f);
            Assert.Less(torque, 2500f);
        }

        [Test]
        public void WheelTorque_ZeroThrottle_ReturnsNegativeEngineBraking()
        {
            var state = new DrivetrainState { currentGear = 0, rpm = 4000f };
            _profile.engineBraking = 50f;
            float torque = Drivetrain.CalculateWheelTorque(0f, state, _profile);
            Assert.Less(torque, 0f, "Engine braking should produce negative torque");
        }

        [Test]
        public void RPMFromWheelSpeed_FirstGear_CalculatesCorrectly()
        {
            // wheelAngularVelocity in rad/s, gear 0 (first), wheelRadius 0.33
            float rpm = Drivetrain.CalculateRPM(50f, 0, _profile);
            // rpm = angVel * gearRatio * finalDrive * 60 / (2*PI)
            // = 50 * 3.6 * 3.7 * 60 / 6.2832 ≈ 6366
            Assert.Greater(rpm, 6000f);
            Assert.Less(rpm, 7000f);
        }

        [Test]
        public void AutoShift_AboveShiftUpRPM_ShiftsUp()
        {
            var state = new DrivetrainState { currentGear = 0, rpm = 6600f };
            int newGear = Drivetrain.AutoShift(state, _profile);
            Assert.AreEqual(1, newGear);
        }

        [Test]
        public void AutoShift_BelowShiftDownRPM_ShiftsDown()
        {
            var state = new DrivetrainState { currentGear = 2, rpm = 2400f };
            int newGear = Drivetrain.AutoShift(state, _profile);
            Assert.AreEqual(1, newGear);
        }

        [Test]
        public void AutoShift_InRange_KeepsGear()
        {
            var state = new DrivetrainState { currentGear = 2, rpm = 4000f };
            int newGear = Drivetrain.AutoShift(state, _profile);
            Assert.AreEqual(2, newGear);
        }

        [Test]
        public void AutoShift_FirstGear_BelowShiftDown_StaysInFirst()
        {
            var state = new DrivetrainState { currentGear = 0, rpm = 2000f };
            int newGear = Drivetrain.AutoShift(state, _profile);
            Assert.AreEqual(0, newGear);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: Test Runner → EditMode → Run All
Expected: 7 failures — `Drivetrain` and `DrivetrainState` do not exist

- [ ] **Step 3: Write implementation**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/Drivetrain.cs
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public struct DrivetrainState
    {
        public int currentGear;  // 0-based index into gearRatios, -1 = reverse
        public float rpm;
    }

    public static class Drivetrain
    {
        /// <summary>
        /// Calculate torque at the driven wheels.
        /// </summary>
        /// <param name="throttle">Throttle input 0-1</param>
        /// <param name="state">Current drivetrain state</param>
        /// <param name="profile">Vehicle profile</param>
        /// <returns>Torque in Nm at the wheel hubs</returns>
        public static float CalculateWheelTorque(float throttle, DrivetrainState state, VehicleProfile profile)
        {
            if (throttle <= 0f)
            {
                // Engine braking: negative torque proportional to RPM
                float brakingTorque = -profile.engineBraking * (state.rpm / profile.maxRPM);
                float gearRatio = state.currentGear >= 0
                    ? profile.gearRatios[state.currentGear]
                    : profile.reverseGearRatio;
                return brakingTorque * gearRatio * profile.finalDriveRatio;
            }

            float engineTorque = profile.torqueCurve.Evaluate(state.rpm) * throttle;
            float gearRatio = state.currentGear >= 0
                ? profile.gearRatios[state.currentGear]
                : profile.reverseGearRatio;

            return engineTorque * gearRatio * profile.finalDriveRatio;
        }

        /// <summary>
        /// Calculate engine RPM from wheel angular velocity.
        /// </summary>
        /// <param name="wheelAngularVelocity">Wheel angular velocity in rad/s</param>
        /// <param name="gear">Current gear index (0-based, -1 for reverse)</param>
        /// <param name="profile">Vehicle profile</param>
        /// <returns>Engine RPM, clamped to idle-max range</returns>
        public static float CalculateRPM(float wheelAngularVelocity, int gear, VehicleProfile profile)
        {
            float gearRatio = gear >= 0
                ? profile.gearRatios[gear]
                : profile.reverseGearRatio;

            float rpm = Mathf.Abs(wheelAngularVelocity) * gearRatio * profile.finalDriveRatio * 60f / (2f * Mathf.PI);
            return Mathf.Clamp(rpm, profile.idleRPM, profile.maxRPM);
        }

        /// <summary>
        /// Split torque between left and right wheels via open differential.
        /// Returns (leftTorque, rightTorque).
        /// </summary>
        /// <param name="totalWheelTorque">Total torque at the axle</param>
        /// <param name="leftAngVel">Left wheel angular velocity</param>
        /// <param name="rightAngVel">Right wheel angular velocity</param>
        public static (float left, float right) OpenDifferential(float totalWheelTorque, float leftAngVel, float rightAngVel)
        {
            // Open diff: torque splits evenly, speed difference is absorbed
            // Both wheels get equal torque regardless of speed difference
            return (totalWheelTorque / 2f, totalWheelTorque / 2f);
        }

        /// <summary>
        /// Determine target gear for automatic transmission.
        /// </summary>
        public static int AutoShift(DrivetrainState state, VehicleProfile profile)
        {
            int gear = state.currentGear;

            if (state.rpm >= profile.shiftUpRPM && gear < profile.gearRatios.Length - 1)
                return gear + 1;

            if (state.rpm <= profile.shiftDownRPM && gear > 0)
                return gear - 1;

            return gear;
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: Test Runner → EditMode → Run All
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/Drivetrain.cs Assets/Tests/EditMode/Vehicle/DrivetrainTests.cs
git commit -m "feat: implement drivetrain with torque calculation and auto-shift"
```

---

### Task 6: Weight Transfer

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/WeightTransfer.cs`
- Create: `Assets/Tests/EditMode/Vehicle/WeightTransferTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Vehicle/WeightTransferTests.cs
using NUnit.Framework;
using LonelyHighway.Vehicle;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode
{
    public class WeightTransferTests
    {
        // Vehicle: 1500kg, wheelbase 2.75m, track 1.56m, CoM height 0.5m
        private readonly float _mass = 1500f;
        private readonly float _wheelbase = 2.75f;
        private readonly float _trackWidth = 1.56f;
        private readonly float _comHeight = 0.5f;
        private readonly float _frontAxleOffset = 1.35f;

        [Test]
        public void StaticLoad_FlatGround_NoAcceleration_DistributesEvenly()
        {
            var loads = WeightTransfer.CalculateWheelLoads(
                _mass, _wheelbase, _trackWidth, _comHeight, _frontAxleOffset,
                longitudinalAccel: 0f, lateralAccel: 0f);

            float totalWeight = _mass * 9.81f;
            float expectedPerWheel = totalWeight / 4f;

            // Front/rear split depends on CoM position, but left/right should be equal
            Assert.AreEqual(loads.frontLeft, loads.frontRight, 0.1f);
            Assert.AreEqual(loads.rearLeft, loads.rearRight, 0.1f);
            Assert.AreEqual(totalWeight, loads.frontLeft + loads.frontRight + loads.rearLeft + loads.rearRight, 1f);
        }

        [Test]
        public void Braking_ShiftsLoadForward()
        {
            var staticLoads = WeightTransfer.CalculateWheelLoads(
                _mass, _wheelbase, _trackWidth, _comHeight, _frontAxleOffset,
                0f, 0f);

            var brakingLoads = WeightTransfer.CalculateWheelLoads(
                _mass, _wheelbase, _trackWidth, _comHeight, _frontAxleOffset,
                longitudinalAccel: -5f, lateralAccel: 0f);

            float staticFront = staticLoads.frontLeft + staticLoads.frontRight;
            float brakingFront = brakingLoads.frontLeft + brakingLoads.frontRight;
            Assert.Greater(brakingFront, staticFront);
        }

        [Test]
        public void RightTurn_ShiftsLoadLeft()
        {
            var staticLoads = WeightTransfer.CalculateWheelLoads(
                _mass, _wheelbase, _trackWidth, _comHeight, _frontAxleOffset,
                0f, 0f);

            // Positive lateral accel = turning right = load shifts left
            var turnLoads = WeightTransfer.CalculateWheelLoads(
                _mass, _wheelbase, _trackWidth, _comHeight, _frontAxleOffset,
                longitudinalAccel: 0f, lateralAccel: 5f);

            float staticLeft = staticLoads.frontLeft + staticLoads.rearLeft;
            float turnLeft = turnLoads.frontLeft + turnLoads.rearLeft;
            Assert.Greater(turnLeft, staticLeft);
        }

        [Test]
        public void TotalLoad_MildAcceleration_PreservesTotalWeight()
        {
            // Use mild acceleration to avoid clamping-to-zero edge case
            var loads = WeightTransfer.CalculateWheelLoads(
                _mass, _wheelbase, _trackWidth, _comHeight, _frontAxleOffset,
                longitudinalAccel: -2f, lateralAccel: 1f);

            float totalWeight = _mass * 9.81f;
            float totalLoad = loads.frontLeft + loads.frontRight + loads.rearLeft + loads.rearRight;
            Assert.AreEqual(totalWeight, totalLoad, 1f);
        }

        [Test]
        public void ExtremeAcceleration_ClampedWheels_TotalLoadMayBeLessThanWeight()
        {
            // Under extreme forces, Mathf.Max(0) clamping means load can be "lost"
            // This is acceptable — it prevents negative normal loads (wheel lift)
            var loads = WeightTransfer.CalculateWheelLoads(
                _mass, _wheelbase, _trackWidth, _comHeight, _frontAxleOffset,
                longitudinalAccel: -8f, lateralAccel: 8f);

            float totalWeight = _mass * 9.81f;
            float totalLoad = loads.frontLeft + loads.frontRight + loads.rearLeft + loads.rearRight;
            Assert.LessOrEqual(totalLoad, totalWeight + 1f);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Expected: 4 failures — `WeightTransfer` does not exist

- [ ] **Step 3: Write implementation**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/WeightTransfer.cs
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public struct WheelLoads
    {
        public float frontLeft;
        public float frontRight;
        public float rearLeft;
        public float rearRight;
    }

    public static class WeightTransfer
    {
        /// <summary>
        /// Calculate dynamic normal load on each wheel based on acceleration forces.
        /// </summary>
        /// <param name="mass">Vehicle mass in kg</param>
        /// <param name="wheelbase">Distance between front and rear axles in meters</param>
        /// <param name="trackWidth">Distance between left and right wheels in meters</param>
        /// <param name="comHeight">Center of mass height in meters</param>
        /// <param name="frontAxleOffset">Front axle distance from CoM in meters</param>
        /// <param name="longitudinalAccel">Forward acceleration in m/s^2 (negative = braking)</param>
        /// <param name="lateralAccel">Lateral acceleration in m/s^2 (positive = right turn)</param>
        public static WheelLoads CalculateWheelLoads(
            float mass, float wheelbase, float trackWidth, float comHeight,
            float frontAxleOffset, float longitudinalAccel, float lateralAccel)
        {
            float gravity = 9.81f;
            float totalWeight = mass * gravity;

            float rearAxleOffset = wheelbase - frontAxleOffset;

            // Static front/rear distribution based on CoM position
            float staticFrontTotal = totalWeight * rearAxleOffset / wheelbase;
            float staticRearTotal = totalWeight * frontAxleOffset / wheelbase;

            // Longitudinal weight transfer (braking shifts load forward)
            float longTransfer = mass * longitudinalAccel * comHeight / wheelbase;

            float frontTotal = staticFrontTotal - longTransfer;
            float rearTotal = staticRearTotal + longTransfer;

            // Lateral weight transfer (right turn shifts load left)
            float latTransferFront = mass * lateralAccel * comHeight / trackWidth * (frontTotal / totalWeight);
            float latTransferRear = mass * lateralAccel * comHeight / trackWidth * (rearTotal / totalWeight);

            return new WheelLoads
            {
                frontLeft = Mathf.Max(0f, frontTotal / 2f + latTransferFront / 2f),
                frontRight = Mathf.Max(0f, frontTotal / 2f - latTransferFront / 2f),
                rearLeft = Mathf.Max(0f, rearTotal / 2f + latTransferRear / 2f),
                rearRight = Mathf.Max(0f, rearTotal / 2f - latTransferRear / 2f),
            };
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: Test Runner → EditMode → Run All
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/WeightTransfer.cs Assets/Tests/EditMode/Vehicle/WeightTransferTests.cs
git commit -m "feat: implement dynamic weight transfer calculation"
```

---

### Task 7: Steering System

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/SteeringSystem.cs`
- Create: `Assets/Tests/EditMode/Vehicle/SteeringSystemTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Vehicle/SteeringSystemTests.cs
using NUnit.Framework;
using LonelyHighway.Vehicle;
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode
{
    public class SteeringSystemTests
    {
        private VehicleProfile _profile;

        [SetUp]
        public void SetUp()
        {
            _profile = ScriptableObject.CreateInstance<VehicleProfile>();
            _profile.maxSteerAngle = 35f;
            _profile.highSpeedSteerMultiplier = 0.3f;
            _profile.steerLimitSpeed = 30f;
        }

        [TearDown]
        public void TearDown()
        {
            Object.DestroyImmediate(_profile);
        }

        [Test]
        public void SteerAngle_AtZeroSpeed_ReturnsFullAngle()
        {
            float angle = SteeringSystem.CalculateSteerAngle(1f, 0f, _profile);
            Assert.AreEqual(35f, angle, 0.1f);
        }

        [Test]
        public void SteerAngle_AtHighSpeed_ReturnsReducedAngle()
        {
            float angle = SteeringSystem.CalculateSteerAngle(1f, 30f, _profile);
            // At steerLimitSpeed: maxSteerAngle * highSpeedSteerMultiplier = 35 * 0.3 = 10.5
            Assert.AreEqual(10.5f, angle, 0.1f);
        }

        [Test]
        public void SteerAngle_AtHalfSpeed_ReturnsMidAngle()
        {
            float angle = SteeringSystem.CalculateSteerAngle(1f, 15f, _profile);
            // Lerp between 35 and 10.5 at t=0.5 = 22.75
            Assert.AreEqual(22.75f, angle, 0.5f);
        }

        [Test]
        public void SteerAngle_NegativeInput_ReturnsNegativeAngle()
        {
            float angle = SteeringSystem.CalculateSteerAngle(-1f, 0f, _profile);
            Assert.AreEqual(-35f, angle, 0.1f);
        }

        [Test]
        public void SelfAlignTorque_AtSlipAngle_ReturnsCenteringForce()
        {
            float torque = SteeringSystem.CalculateSelfAlignTorque(0.1f, 5000f, trailLength: 0.03f);
            // Simplified: torque = normalLoad * trail * sin(slipAngle)
            Assert.Greater(torque, 0f);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Expected: 5 failures

- [ ] **Step 3: Write implementation**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/SteeringSystem.cs
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public static class SteeringSystem
    {
        /// <summary>
        /// Calculate effective steering angle based on input and vehicle speed.
        /// </summary>
        /// <param name="steerInput">Steering input -1 to 1</param>
        /// <param name="speed">Vehicle speed in m/s</param>
        /// <param name="profile">Vehicle profile</param>
        /// <returns>Steering angle in degrees</returns>
        public static float CalculateSteerAngle(float steerInput, float speed, VehicleProfile profile)
        {
            float t = Mathf.Clamp01(speed / profile.steerLimitSpeed);
            float maxAngle = Mathf.Lerp(profile.maxSteerAngle, profile.maxSteerAngle * profile.highSpeedSteerMultiplier, t);
            return steerInput * maxAngle;
        }

        /// <summary>
        /// Calculate self-aligning torque that returns steering to center.
        /// </summary>
        /// <param name="slipAngle">Tire slip angle in radians</param>
        /// <param name="normalLoad">Normal force on front axle in Newtons</param>
        /// <param name="trailLength">Pneumatic trail length in meters</param>
        /// <returns>Self-aligning torque in Nm</returns>
        public static float CalculateSelfAlignTorque(float slipAngle, float normalLoad, float trailLength)
        {
            return normalLoad * trailLength * Mathf.Sin(slipAngle);
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/SteeringSystem.cs Assets/Tests/EditMode/Vehicle/SteeringSystemTests.cs
git commit -m "feat: implement speed-sensitive steering with self-aligning torque"
```

---

### Task 8: Damage System

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/DamageSystem.cs`
- Create: `Assets/Tests/EditMode/Vehicle/DamageSystemTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// Assets/Tests/EditMode/Vehicle/DamageSystemTests.cs
using NUnit.Framework;
using LonelyHighway.Vehicle;
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode
{
    public class DamageSystemTests
    {
        private VehicleProfile _profile;

        [SetUp]
        public void SetUp()
        {
            _profile = ScriptableObject.CreateInstance<VehicleProfile>();
            _profile.collisionDamageThreshold = 3f;
            _profile.alignmentDriftRate = 0.5f;
            _profile.engineStutterChance = 0.1f;
            _profile.passiveRecoveryRate = 0.05f;
        }

        [TearDown]
        public void TearDown()
        {
            Object.DestroyImmediate(_profile);
        }

        [Test]
        public void NewDamageState_AllPanelsFullHealth()
        {
            var state = new DamageState();
            Assert.AreEqual(100f, state.GetPanelHealth(DamagePanel.FrontLeft));
            Assert.AreEqual(100f, state.GetPanelHealth(DamagePanel.Rear));
        }

        [Test]
        public void ApplyImpact_BelowThreshold_NoDamage()
        {
            var state = new DamageState();
            state.ApplyImpact(DamagePanel.FrontLeft, impactForce: 2f, _profile);
            Assert.AreEqual(100f, state.GetPanelHealth(DamagePanel.FrontLeft));
        }

        [Test]
        public void ApplyImpact_AboveThreshold_ReducesHealth()
        {
            var state = new DamageState();
            state.ApplyImpact(DamagePanel.FrontLeft, impactForce: 10f, _profile);
            Assert.Less(state.GetPanelHealth(DamagePanel.FrontLeft), 100f);
        }

        [Test]
        public void ApplyImpact_HealthNeverBelowZero()
        {
            var state = new DamageState();
            state.ApplyImpact(DamagePanel.FrontLeft, impactForce: 10000f, _profile);
            Assert.GreaterOrEqual(state.GetPanelHealth(DamagePanel.FrontLeft), 0f);
        }

        [Test]
        public void AlignmentDrift_IncreasesWithFrontDamage()
        {
            var state = new DamageState();
            Assert.AreEqual(0f, state.GetAlignmentDrift(_profile), 0.01f);

            state.ApplyImpact(DamagePanel.FrontLeft, 20f, _profile);
            Assert.Greater(Mathf.Abs(state.GetAlignmentDrift(_profile)), 0f);
        }

        [Test]
        public void PassiveRecovery_RestoresHealth()
        {
            var state = new DamageState();
            state.ApplyImpact(DamagePanel.FrontLeft, 10f, _profile);
            float damagedHealth = state.GetPanelHealth(DamagePanel.FrontLeft);

            state.ApplyPassiveRecovery(60f, _profile); // 60 seconds
            Assert.Greater(state.GetPanelHealth(DamagePanel.FrontLeft), damagedHealth);
        }

        [Test]
        public void FullRepair_RestoresAllPanels()
        {
            var state = new DamageState();
            state.ApplyImpact(DamagePanel.FrontLeft, 20f, _profile);
            state.ApplyImpact(DamagePanel.Rear, 15f, _profile);

            state.FullRepair();
            Assert.AreEqual(100f, state.GetPanelHealth(DamagePanel.FrontLeft));
            Assert.AreEqual(100f, state.GetPanelHealth(DamagePanel.Rear));
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Expected: 7 failures

- [ ] **Step 3: Write implementation**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/DamageSystem.cs
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public enum DamagePanel
    {
        FrontLeft,
        FrontRight,
        RearLeft,
        RearRight,
        Front,
        Rear
    }

    public class DamageState
    {
        private readonly float[] _panelHealth;
        private const int PanelCount = 6;

        public DamageState()
        {
            _panelHealth = new float[PanelCount];
            FullRepair();
        }

        public float GetPanelHealth(DamagePanel panel)
        {
            return _panelHealth[(int)panel];
        }

        /// <summary>
        /// Apply collision damage to a panel.
        /// </summary>
        /// <param name="panel">Which panel was hit</param>
        /// <param name="impactForce">Collision impulse magnitude in kN</param>
        /// <param name="profile">Vehicle profile for threshold</param>
        public void ApplyImpact(DamagePanel panel, float impactForce, VehicleProfile profile)
        {
            if (impactForce <= profile.collisionDamageThreshold)
                return;

            float damage = (impactForce - profile.collisionDamageThreshold) * 2f;
            _panelHealth[(int)panel] = Mathf.Max(0f, _panelHealth[(int)panel] - damage);
        }

        /// <summary>
        /// Get steering alignment drift caused by front panel damage.
        /// Returns a small angle offset in degrees.
        /// </summary>
        public float GetAlignmentDrift(VehicleProfile profile)
        {
            float leftDamage = 100f - _panelHealth[(int)DamagePanel.FrontLeft];
            float rightDamage = 100f - _panelHealth[(int)DamagePanel.FrontRight];
            return (leftDamage - rightDamage) * profile.alignmentDriftRate / 100f;
        }

        /// <summary>
        /// Get engine stutter factor (0 = no stutter, 1 = maximum stutter).
        /// Based on average front panel damage.
        /// </summary>
        public float GetEngineStutterFactor(VehicleProfile profile)
        {
            float avgFrontDamage = ((100f - _panelHealth[(int)DamagePanel.FrontLeft])
                + (100f - _panelHealth[(int)DamagePanel.FrontRight])
                + (100f - _panelHealth[(int)DamagePanel.Front])) / 300f;
            return avgFrontDamage * profile.engineStutterChance;
        }

        /// <summary>
        /// Get suspension sag multiplier for a specific corner.
        /// Returns a value 0-1 where 0 = full sag, 1 = no sag.
        /// Applied as a multiplier to rest length.
        /// </summary>
        public float GetSuspensionSagMultiplier(DamagePanel panel, VehicleProfile profile)
        {
            float damage = (100f - _panelHealth[(int)panel]) / 100f;
            return 1f - (damage * profile.suspensionSagRate);
        }

        /// <summary>
        /// Apply passive recovery over time.
        /// </summary>
        /// <param name="deltaTime">Time elapsed in seconds</param>
        /// <param name="profile">Vehicle profile for recovery rate</param>
        public void ApplyPassiveRecovery(float deltaTime, VehicleProfile profile)
        {
            float recovery = profile.passiveRecoveryRate * deltaTime;
            for (int i = 0; i < PanelCount; i++)
            {
                _panelHealth[i] = Mathf.Min(100f, _panelHealth[i] + recovery);
            }
        }

        /// <summary>
        /// Full instant repair (garage visit).
        /// </summary>
        public void FullRepair()
        {
            for (int i = 0; i < PanelCount; i++)
                _panelHealth[i] = 100f;
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/DamageSystem.cs Assets/Tests/EditMode/Vehicle/DamageSystemTests.cs
git commit -m "feat: implement collision damage system with panel health and recovery"
```

---

### Task 9: Wheel Raycast

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/WheelRaycast.cs`

This is a MonoBehaviour that uses Physics.Raycast — cannot be unit tested in EditMode. Tested via integration tests in Task 12.

- [ ] **Step 1: Write implementation**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/WheelRaycast.cs
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public struct WheelHitInfo
    {
        public bool isGrounded;
        public float compression;         // meters, 0 = rest, positive = compressed
        public float compressionVelocity;  // m/s, negative = compressing
        public Vector3 contactPoint;
        public Vector3 contactNormal;
        public SurfaceType surfaceType;
        public Collider hitCollider;
    }

    public class WheelRaycast : MonoBehaviour
    {
        [HideInInspector] public WheelHitInfo hitInfo;

        private float _previousCompression;

        /// <summary>
        /// Cast ray downward from this wheel's position.
        /// Call from VehicleController.FixedUpdate.
        /// </summary>
        public WheelHitInfo CastWheel(float restLength, float maxTravel, float wheelRadius)
        {
            float rayLength = restLength + maxTravel + wheelRadius;
            var origin = transform.position;

            if (Physics.Raycast(origin, -transform.up, out RaycastHit hit, rayLength))
            {
                float springLength = hit.distance - wheelRadius;
                float compression = restLength - springLength;

                float compressionVelocity = (compression - _previousCompression) / Time.fixedDeltaTime;
                _previousCompression = compression;

                // Determine surface type from SurfaceIdentifier component on collider
                SurfaceType surface = SurfaceType.DryAsphalt;
                var surfaceId = hit.collider.GetComponent<SurfaceIdentifier>();
                if (surfaceId != null) surface = surfaceId.surfaceType;

                hitInfo = new WheelHitInfo
                {
                    isGrounded = true,
                    compression = compression,
                    compressionVelocity = compressionVelocity,
                    contactPoint = hit.point,
                    contactNormal = hit.normal,
                    surfaceType = surface,
                    hitCollider = hit.collider
                };
            }
            else
            {
                _previousCompression = 0f;
                hitInfo = new WheelHitInfo { isGrounded = false };
            }

            return hitInfo;
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Open Unity — no compile errors.

- [ ] **Step 3: Write SurfaceIdentifier**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/SurfaceIdentifier.cs
using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public class SurfaceIdentifier : MonoBehaviour
    {
        public SurfaceType surfaceType = SurfaceType.DryAsphalt;
    }
}
```

- [ ] **Step 4: Write GarageInteraction stub**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/GarageInteraction.cs
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    [RequireComponent(typeof(BoxCollider))]
    public class GarageInteraction : MonoBehaviour
    {
        private void Awake()
        {
            // Ensure trigger collider
            var col = GetComponent<BoxCollider>();
            col.isTrigger = true;
        }

        private void OnTriggerEnter(Collider other)
        {
            var controller = other.GetComponentInParent<VehicleController>();
            if (controller == null) return;

            // Full repair
            controller.Damage.FullRepair();
            // TODO: fade-to-black UI transition (wired in UI plan)
            Debug.Log("Vehicle repaired at garage");
        }
    }
}
```

- [ ] **Step 5: Add SurfaceIdentifier to test track planes**

In TestTrack scene, add `SurfaceIdentifier` component to each ground plane:
- Main plane: surfaceType = DryAsphalt
- Wet section: surfaceType = WetAsphalt
- Painted line strip: surfaceType = PaintedLine

- [ ] **Step 6: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/WheelRaycast.cs Assets/LonelyHighway/Scripts/Vehicle/SurfaceIdentifier.cs Assets/LonelyHighway/Scripts/Vehicle/GarageInteraction.cs
git commit -m "feat: implement wheel raycast, surface identifier, and garage interaction stub"
```

---

### Task 10: Vehicle Input

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/VehicleInput.cs`
- Create: `Assets/LonelyHighway/Data/VehicleInputActions.inputactions`

- [ ] **Step 1: Create Input Actions asset**

In Unity Editor: right-click `Assets/LonelyHighway/Data/` → Create → Input Actions → name "VehicleInputActions"

Open the asset and define these action maps and actions:

**Action Map: Vehicle**
| Action | Type | Bindings |
|--------|------|----------|
| Steer | Value (Axis) | Gamepad Left Stick X, Keyboard A/D |
| Throttle | Value (Axis) | Gamepad Right Trigger, Keyboard W |
| Brake | Value (Axis) | Gamepad Left Trigger, Keyboard S |
| Handbrake | Button | Gamepad A/X, Keyboard Space |
| ShiftUp | Button | Gamepad Right Bumper, Keyboard E |
| ShiftDown | Button | Gamepad Left Bumper, Keyboard Q |
| Horn | Button | Gamepad Y/Triangle, Keyboard H |
| Headlights | Button | Keyboard L |
| TurnSignalLeft | Button | Keyboard Z |
| TurnSignalRight | Button | Keyboard X |
| Wipers | Button | Keyboard V |
| CycleCamera | Button | Keyboard C |

Also add bindings for steering wheel peripherals:
- Steer: add binding for Steering Wheel Axis (HID axis 0)
- Throttle: add binding for Pedal Axis (HID axis 1)
- Brake: add binding for Pedal Axis (HID axis 2)

This ensures Logitech G29/G920 and Thrustmaster wheels work out of the box via Unity Input System's HID support. Force feedback integration is deferred to a later plan.

Check "Generate C# Class", set class name to `VehicleInputActions`, namespace `LonelyHighway.Vehicle`.

- [ ] **Step 2: Write VehicleInput wrapper**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/VehicleInput.cs
using UnityEngine;
using UnityEngine.InputSystem;

namespace LonelyHighway.Vehicle
{
    public class VehicleInput : MonoBehaviour
    {
        // Continuous inputs (safe to read any time)
        public float Steer { get; private set; }
        public float Throttle { get; private set; }
        public float Brake { get; private set; }
        public bool Handbrake { get; private set; }
        public bool Horn { get; private set; }

        // Buffered button presses — captured in Update, consumed in FixedUpdate
        // This prevents missed presses when Update runs multiple times per FixedUpdate
        public bool ShiftUp { get; private set; }
        public bool ShiftDown { get; private set; }
        public bool HeadlightsToggle { get; private set; }
        public bool TurnSignalLeft { get; private set; }
        public bool TurnSignalRight { get; private set; }
        public bool WipersToggle { get; private set; }
        public bool CycleCamera { get; private set; }

        private VehicleInputActions _actions;

        // Buffers for one-shot presses
        private bool _shiftUpBuffer, _shiftDownBuffer;
        private bool _headlightsBuffer, _turnLeftBuffer, _turnRightBuffer;
        private bool _wipersBuffer, _cameraCycleBuffer;

        private void OnEnable()
        {
            _actions = new VehicleInputActions();
            _actions.Vehicle.Enable();
        }

        private void OnDisable()
        {
            _actions.Vehicle.Disable();
            _actions.Dispose();
        }

        private void Update()
        {
            var v = _actions.Vehicle;
            Steer = v.Steer.ReadValue<float>();
            Throttle = Mathf.Clamp01(v.Throttle.ReadValue<float>());
            Brake = Mathf.Clamp01(v.Brake.ReadValue<float>());
            Handbrake = v.Handbrake.IsPressed();
            Horn = v.Horn.IsPressed();

            // Buffer one-shot presses (sticky until consumed)
            _shiftUpBuffer |= v.ShiftUp.WasPressedThisFrame();
            _shiftDownBuffer |= v.ShiftDown.WasPressedThisFrame();
            _headlightsBuffer |= v.Headlights.WasPressedThisFrame();
            _turnLeftBuffer |= v.TurnSignalLeft.WasPressedThisFrame();
            _turnRightBuffer |= v.TurnSignalRight.WasPressedThisFrame();
            _wipersBuffer |= v.Wipers.WasPressedThisFrame();
            _cameraCycleBuffer |= v.CycleCamera.WasPressedThisFrame();
        }

        /// <summary>
        /// Call from FixedUpdate to consume buffered presses.
        /// </summary>
        public void ConsumeBufferedInputs()
        {
            ShiftUp = _shiftUpBuffer;
            ShiftDown = _shiftDownBuffer;
            HeadlightsToggle = _headlightsBuffer;
            TurnSignalLeft = _turnLeftBuffer;
            TurnSignalRight = _turnRightBuffer;
            WipersToggle = _wipersBuffer;
            CycleCamera = _cameraCycleBuffer;

            _shiftUpBuffer = _shiftDownBuffer = false;
            _headlightsBuffer = _turnLeftBuffer = _turnRightBuffer = false;
            _wipersBuffer = _cameraCycleBuffer = false;
        }
    }
}
```

- [ ] **Step 3: Verify it compiles**

Open Unity — no compile errors.

- [ ] **Step 4: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/VehicleInput.cs Assets/LonelyHighway/Data/VehicleInputActions*
git commit -m "feat: implement vehicle input via Unity Input System"
```

---

### Task 11: Vehicle Controller (Integration)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/VehicleController.cs`

This is the top-level MonoBehaviour that wires all subsystems together in FixedUpdate.

- [ ] **Step 1: Write VehicleController**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/VehicleController.cs
using LonelyHighway.Data;
using UnityEngine;
using System.Collections.Generic;

namespace LonelyHighway.Vehicle
{
    public class VehicleController : MonoBehaviour
    {
        [Header("Configuration")]
        public VehicleProfile profile;
        public TireProfile defaultTireProfile;
        public List<SurfaceTireMapping> surfaceTireMappings = new();

        [Header("Wheel Transforms (set in inspector)")]
        public WheelRaycast wheelFL;
        public WheelRaycast wheelFR;
        public WheelRaycast wheelRL;
        public WheelRaycast wheelRR;

        [Header("Wheel Mesh Transforms (visual only)")]
        public Transform wheelMeshFL;
        public Transform wheelMeshFR;
        public Transform wheelMeshRL;
        public Transform wheelMeshRR;

        // Public state for UI / audio / other systems
        public float SpeedKmh => _rb.linearVelocity.magnitude * 3.6f;
        public float EngineRPM => _drivetrainState.rpm;
        public int CurrentGear => _drivetrainState.currentGear;
        public DamageState Damage => _damageState;

        private Rigidbody _rb;
        private VehicleInput _input;
        private DrivetrainState _drivetrainState;
        private DamageState _damageState;
        private bool _autoTransmission = true;

        // Per-wheel working data
        private float[] _wheelAngularVelocity = new float[4];
        private Vector3 _previousLocalVelocity;

        [System.Serializable]
        public struct SurfaceTireMapping
        {
            public SurfaceType surface;
            public TireProfile tireProfile;
        }

        private void Awake()
        {
            _rb = GetComponent<Rigidbody>();
            _input = GetComponent<VehicleInput>();
            _damageState = new DamageState();
            _drivetrainState = new DrivetrainState { currentGear = 0, rpm = profile.idleRPM };

            _rb.mass = profile.mass;
            _rb.centerOfMass = profile.centerOfMass;
        }

        private void FixedUpdate()
        {
            _input.ConsumeBufferedInputs();
            float dt = Time.fixedDeltaTime;

            // 1. Raycast all wheels
            var hits = new WheelHitInfo[4];
            WheelRaycast[] wheels = { wheelFL, wheelFR, wheelRL, wheelRR };
            DamagePanel[] wheelPanels = { DamagePanel.FrontLeft, DamagePanel.FrontRight, DamagePanel.RearLeft, DamagePanel.RearRight };
            for (int i = 0; i < 4; i++)
            {
                float sagMultiplier = _damageState.GetSuspensionSagMultiplier(wheelPanels[i], profile);
                float effectiveRestLength = profile.restLength * sagMultiplier;
                hits[i] = wheels[i].CastWheel(effectiveRestLength, profile.maxTravel, profile.wheelRadius);
            }

            // 2. Suspension forces
            float[] compressions = new float[4];
            for (int i = 0; i < 4; i++)
            {
                if (!hits[i].isGrounded) continue;

                compressions[i] = hits[i].compression;
                float springForce = SuspensionSystem.CalculateSpringForce(
                    hits[i].compression, hits[i].compressionVelocity,
                    profile.springRate, profile.damperRate);

                _rb.AddForceAtPosition(
                    hits[i].contactNormal * springForce,
                    wheels[i].transform.position);
            }

            // Anti-roll bars
            float antiRollFront = SuspensionSystem.CalculateAntiRollForce(
                compressions[0], compressions[1], profile.antiRollBarStiffness);
            float antiRollRear = SuspensionSystem.CalculateAntiRollForce(
                compressions[2], compressions[3], profile.antiRollBarStiffness);

            // Anti-roll: positive antiRollFront means left is more compressed,
            // so push left up (positive) and right down (negative)
            if (hits[0].isGrounded)
                _rb.AddForceAtPosition(transform.up * antiRollFront, wheels[0].transform.position);
            if (hits[1].isGrounded)
                _rb.AddForceAtPosition(-transform.up * antiRollFront, wheels[1].transform.position);
            if (hits[2].isGrounded)
                _rb.AddForceAtPosition(transform.up * antiRollRear, wheels[2].transform.position);
            if (hits[3].isGrounded)
                _rb.AddForceAtPosition(-transform.up * antiRollRear, wheels[3].transform.position);

            // 3. Weight transfer
            Vector3 localVelocity = transform.InverseTransformDirection(_rb.linearVelocity);
            Vector3 localAccel = (localVelocity - _previousLocalVelocity) / dt;
            _previousLocalVelocity = localVelocity;

            var wheelLoads = WeightTransfer.CalculateWheelLoads(
                profile.mass,
                profile.frontAxleOffset + profile.rearAxleOffset,
                profile.trackHalfWidth * 2f,
                profile.centerOfMass.y,
                profile.frontAxleOffset,
                localAccel.z, localAccel.x);
            float[] loads = { wheelLoads.frontLeft, wheelLoads.frontRight, wheelLoads.rearLeft, wheelLoads.rearRight };

            // 4. Steering
            float steerAngle = SteeringSystem.CalculateSteerAngle(
                _input.Steer, _rb.linearVelocity.magnitude, profile);
            steerAngle += _damageState.GetAlignmentDrift(profile);

            // 5. Tire forces
            float speed = _rb.linearVelocity.magnitude;
            for (int i = 0; i < 4; i++)
            {
                if (!hits[i].isGrounded) continue;

                TireProfile tireProfile = GetTireProfile(hits[i].surfaceType);

                // Calculate slip angle
                Vector3 wheelWorldForward = wheels[i].transform.forward;
                Vector3 wheelWorldRight = wheels[i].transform.right;

                // Apply steering to front wheels
                if (i < 2)
                {
                    float steerRad = steerAngle * Mathf.Deg2Rad;
                    wheelWorldForward = Quaternion.AngleAxis(steerAngle, transform.up) * wheels[i].transform.forward;
                    wheelWorldRight = Quaternion.AngleAxis(steerAngle, transform.up) * wheels[i].transform.right;
                }

                Vector3 pointVelocity = _rb.GetPointVelocity(wheels[i].transform.position);
                float forwardSpeed = Vector3.Dot(pointVelocity, wheelWorldForward);
                float sidewaysSpeed = Vector3.Dot(pointVelocity, wheelWorldRight);

                float slipAngle = (speed > 0.5f) ? Mathf.Atan2(sidewaysSpeed, Mathf.Abs(forwardSpeed)) : 0f;

                // Calculate slip ratio
                float wheelSpeed = _wheelAngularVelocity[i] * profile.wheelRadius;
                float slipRatio = (speed > 0.5f)
                    ? (wheelSpeed - forwardSpeed) / Mathf.Max(Mathf.Abs(forwardSpeed), Mathf.Abs(wheelSpeed), 0.5f)
                    : 0f;

                // Pacejka forces
                float lateralForce = PacejkaTireModel.CalculateLateralForce(slipAngle, loads[i], tireProfile);
                float longitudinalForce = PacejkaTireModel.CalculateLongitudinalForce(slipRatio, loads[i], tireProfile);

                // Apply forces at contact point
                Vector3 totalForce = wheelWorldForward * longitudinalForce - wheelWorldRight * lateralForce;
                _rb.AddForceAtPosition(totalForce, hits[i].contactPoint);

                // Update wheel angular velocity
                float driveTorque = 0f;
                if (i >= 2) // Rear-wheel drive
                {
                    float totalAxleTorque = Drivetrain.CalculateWheelTorque(_input.Throttle, _drivetrainState, profile);
                    var (leftT, rightT) = Drivetrain.OpenDifferential(
                        totalAxleTorque, _wheelAngularVelocity[2], _wheelAngularVelocity[3]);
                    driveTorque = (i == 2) ? leftT : rightT;
                }

                float brakeTorque = _input.Brake * profile.maxBrakeForce *
                    (i < 2 ? profile.brakeBias : 1f - profile.brakeBias);

                if (_input.Handbrake && i >= 2)
                    brakeTorque += profile.handbrakeForce;

                float netTorque = driveTorque - Mathf.Sign(_wheelAngularVelocity[i]) * brakeTorque;
                float wheelInertia = profile.wheelMass * profile.wheelRadius * profile.wheelRadius;
                _wheelAngularVelocity[i] += (netTorque / wheelInertia) * dt;
            }

            // 6. Drivetrain update
            float avgRearWheelAngVel = (_wheelAngularVelocity[2] + _wheelAngularVelocity[3]) / 2f;
            _drivetrainState.rpm = Drivetrain.CalculateRPM(avgRearWheelAngVel, _drivetrainState.currentGear, profile);

            if (_autoTransmission)
            {
                _drivetrainState.currentGear = Drivetrain.AutoShift(_drivetrainState, profile);
            }

            if (_input.ShiftUp && _drivetrainState.currentGear < profile.gearRatios.Length - 1)
            {
                _drivetrainState.currentGear++;
                _autoTransmission = false;
            }
            if (_input.ShiftDown && _drivetrainState.currentGear > -1)
            {
                _drivetrainState.currentGear--;
                _autoTransmission = false;
            }

            // 7. Damage recovery
            _damageState.ApplyPassiveRecovery(dt, profile);

            // 8. Update wheel meshes
            UpdateWheelMeshes(steerAngle);
        }

        private void OnCollisionEnter(Collision collision)
        {
            float impulse = collision.impulse.magnitude / 1000f; // convert to kN
            Vector3 localContact = transform.InverseTransformPoint(collision.GetContact(0).point);

            DamagePanel panel = DeterminePanel(localContact);
            _damageState.ApplyImpact(panel, impulse, profile);
        }

        private DamagePanel DeterminePanel(Vector3 localPoint)
        {
            bool isFront = localPoint.z > 0f;
            bool isLeft = localPoint.x < 0f;
            bool isCenter = Mathf.Abs(localPoint.x) < 0.3f;

            if (isCenter)
                return isFront ? DamagePanel.Front : DamagePanel.Rear;

            if (isFront)
                return isLeft ? DamagePanel.FrontLeft : DamagePanel.FrontRight;
            else
                return isLeft ? DamagePanel.RearLeft : DamagePanel.RearRight;
        }

        private TireProfile GetTireProfile(SurfaceType surface)
        {
            foreach (var mapping in surfaceTireMappings)
            {
                if (mapping.surface == surface)
                    return mapping.tireProfile;
            }
            return defaultTireProfile;
        }

        private void UpdateWheelMeshes(float steerAngle)
        {
            Transform[] meshes = { wheelMeshFL, wheelMeshFR, wheelMeshRL, wheelMeshRR };
            WheelRaycast[] wheels = { wheelFL, wheelFR, wheelRL, wheelRR };

            for (int i = 0; i < 4; i++)
            {
                if (meshes[i] == null) continue;

                // Position: wheel anchor minus suspension compression
                Vector3 pos = wheels[i].transform.position - wheels[i].transform.up *
                    (profile.restLength - wheels[i].hitInfo.compression + profile.wheelRadius);

                if (!wheels[i].hitInfo.isGrounded)
                    pos = wheels[i].transform.position - wheels[i].transform.up *
                        (profile.restLength + profile.maxTravel + profile.wheelRadius);

                meshes[i].position = pos;

                // Steering rotation on front wheels
                if (i < 2)
                {
                    meshes[i].localRotation = Quaternion.Euler(0f, steerAngle, 0f);
                }

                // Spin rotation
                float spinDegrees = _wheelAngularVelocity[i] * Mathf.Rad2Deg * Time.fixedDeltaTime;
                meshes[i].Rotate(Vector3.right, spinDegrees, Space.Self);
            }
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Open Unity — no compile errors.

- [ ] **Step 3: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/VehicleController.cs
git commit -m "feat: implement VehicleController integrating all physics subsystems"
```

---

### Task 12: Vehicle Prefab & Integration Test

**Files:**
- Create: `Assets/LonelyHighway/Prefabs/PlayerVehicle.prefab`
- Create: `Assets/Tests/PlayMode/Vehicle/VehicleIntegrationTests.cs`
- Create: `Assets/Tests/PlayMode/Vehicle/SurfaceGripTests.cs`

- [ ] **Step 1: Build vehicle prefab in scene**

In TestTrack scene:
1. Create empty GameObject "PlayerVehicle"
2. Add components: Rigidbody, VehicleController, VehicleInput
3. Add a box collider as vehicle body (scale ~4.6m x 1.4m x 1.8m)
4. Create 4 child empty GameObjects for wheel anchors: "WheelAnchorFL", "WheelAnchorFR", "WheelAnchorRL", "WheelAnchorRR"
   - Position FL: (-0.78, 0.35, 1.35)
   - Position FR: (0.78, 0.35, 1.35)
   - Position RL: (-0.78, 0.35, -1.4)
   - Position RR: (0.78, 0.35, -1.4)
5. Add WheelRaycast component to each anchor
6. Create 4 cylinder primitives as wheel meshes (scale 0.33 radius)
7. Assign references in VehicleController inspector:
   - profile → BYD-Qin asset
   - defaultTireProfile → DryAsphalt asset
   - surfaceTireMappings → add WetAsphalt and PaintedLine entries
   - wheel raycast references → the 4 anchors
   - wheel mesh references → the 4 cylinders
8. Drag to Prefabs folder to create prefab

- [ ] **Step 2: Add Unity tags for surface types**

In Edit > Project Settings > Tags and Layers, add tags:
- DryAsphalt, WetAsphalt, PaintedLine, Concrete, Gravel, Grass

Tag the test track planes with their respective surface tags.

- [ ] **Step 3: Write integration tests**

```csharp
// Assets/Tests/PlayMode/Vehicle/VehicleIntegrationTests.cs
using System.Collections;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;
using LonelyHighway.Vehicle;
using LonelyHighway.Data;

namespace LonelyHighway.Tests.PlayMode
{
    public class VehicleIntegrationTests
    {
        private GameObject _vehicle;
        private VehicleController _controller;

        [UnitySetUp]
        public IEnumerator SetUp()
        {
            // Create ground
            var ground = GameObject.CreatePrimitive(PrimitiveType.Plane);
            ground.transform.localScale = new Vector3(20f, 1f, 20f);
            ground.tag = "DryAsphalt";

            // Create vehicle
            _vehicle = new GameObject("TestVehicle");
            var rb = _vehicle.AddComponent<Rigidbody>();
            _vehicle.transform.position = new Vector3(0f, 1f, 0f);

            // Add box collider
            var col = _vehicle.AddComponent<BoxCollider>();
            col.size = new Vector3(1.8f, 1.4f, 4.6f);

            // Add wheel anchors
            var anchors = new WheelRaycast[4];
            Vector3[] positions = {
                new(-0.78f, 0.35f, 1.35f),
                new(0.78f, 0.35f, 1.35f),
                new(-0.78f, 0.35f, -1.4f),
                new(0.78f, 0.35f, -1.4f)
            };
            for (int i = 0; i < 4; i++)
            {
                var anchor = new GameObject($"Wheel{i}");
                anchor.transform.SetParent(_vehicle.transform);
                anchor.transform.localPosition = positions[i];
                anchors[i] = anchor.AddComponent<WheelRaycast>();
            }

            // Create profiles
            var vehicleProfile = ScriptableObject.CreateInstance<VehicleProfile>();
            var tireProfile = ScriptableObject.CreateInstance<TireProfile>();

            // Add controller
            _vehicle.AddComponent<VehicleInput>();
            _controller = _vehicle.AddComponent<VehicleController>();
            _controller.profile = vehicleProfile;
            _controller.defaultTireProfile = tireProfile;
            _controller.wheelFL = anchors[0];
            _controller.wheelFR = anchors[1];
            _controller.wheelRL = anchors[2];
            _controller.wheelRR = anchors[3];

            // Let physics settle
            yield return new WaitForFixedUpdate();
            yield return new WaitForFixedUpdate();
            yield return new WaitForFixedUpdate();
        }

        [UnityTest]
        public IEnumerator Vehicle_OnGround_DoesNotFallThrough()
        {
            yield return new WaitForSeconds(1f);
            Assert.Greater(_vehicle.transform.position.y, 0f, "Vehicle should rest on ground, not fall through");
        }

        [UnityTest]
        public IEnumerator Vehicle_SpeedStartsAtZero()
        {
            yield return new WaitForSeconds(0.5f);
            Assert.Less(_controller.SpeedKmh, 1f, "Vehicle should start nearly stationary");
        }

        [UnityTearDown]
        public IEnumerator TearDown()
        {
            Object.Destroy(_vehicle);
            // Clean up ground
            var ground = GameObject.Find("Plane");
            if (ground) Object.Destroy(ground);
            yield return null;
        }
    }
}
```

- [ ] **Step 4: Write SurfaceGripTests**

```csharp
// Assets/Tests/PlayMode/Vehicle/SurfaceGripTests.cs
using System.Collections;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;
using LonelyHighway.Vehicle;
using LonelyHighway.Data;

namespace LonelyHighway.Tests.PlayMode
{
    public class SurfaceGripTests
    {
        [UnityTest]
        public IEnumerator DryAsphalt_HigherGrip_ThanWetAsphalt()
        {
            // Create two ground planes with different surfaces
            var dryGround = GameObject.CreatePrimitive(PrimitiveType.Plane);
            dryGround.transform.localScale = new Vector3(20f, 1f, 20f);
            dryGround.transform.position = new Vector3(0f, 0f, 0f);
            var drySurface = dryGround.AddComponent<SurfaceIdentifier>();
            drySurface.surfaceType = SurfaceType.DryAsphalt;

            var wetGround = GameObject.CreatePrimitive(PrimitiveType.Plane);
            wetGround.transform.localScale = new Vector3(20f, 1f, 20f);
            wetGround.transform.position = new Vector3(50f, 0f, 0f);
            var wetSurface = wetGround.AddComponent<SurfaceIdentifier>();
            wetSurface.surfaceType = SurfaceType.WetAsphalt;

            // Create tire profiles
            var dryProfile = ScriptableObject.CreateInstance<TireProfile>();
            // defaults: D = 1.0

            var wetProfile = ScriptableObject.CreateInstance<TireProfile>();
            wetProfile.lateralD = 0.7f;
            wetProfile.longitudinalD = 0.7f;

            // Verify wet grip is lower
            float dryForce = PacejkaTireModel.CalculateLateralForce(0.1f, 5000f, dryProfile);
            float wetForce = PacejkaTireModel.CalculateLateralForce(0.1f, 5000f, wetProfile);

            Assert.Greater(dryForce, wetForce, "Dry asphalt should provide more grip than wet");

            Object.Destroy(dryGround);
            Object.Destroy(wetGround);
            Object.DestroyImmediate(dryProfile);
            Object.DestroyImmediate(wetProfile);
            yield return null;
        }
    }
}
```

- [ ] **Step 5: Run PlayMode tests**

Run: Test Runner → PlayMode → Run All
Expected: All PASS

- [ ] **Step 6: Manual drive test**

Enter Play mode in TestTrack scene. Use WASD to drive:
- W = throttle, S = brake, A/D = steer, Space = handbrake
- Verify: car moves forward, steers, brakes, doesn't jitter or fall through ground
- Verify: speed-sensitive steering feels correct (less steering at speed)

- [ ] **Step 7: Commit**

```bash
git add Assets/LonelyHighway/Prefabs/ Assets/Tests/PlayMode/ Assets/LonelyHighway/Scenes/TestTrack.unity
git commit -m "feat: assemble vehicle prefab and add integration and surface grip tests"
```

---

### Task 13: Vehicle Audio (Placeholder)

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Vehicle/VehicleAudio.cs`

Basic audio using Unity's built-in audio. FMOD/Wwise integration comes in the Environment plan.

- [ ] **Step 1: Write placeholder audio**

```csharp
// Assets/LonelyHighway/Scripts/Vehicle/VehicleAudio.cs
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    [RequireComponent(typeof(VehicleController))]
    public class VehicleAudio : MonoBehaviour
    {
        [Header("Audio Sources")]
        public AudioSource engineSource;
        public AudioSource tireSource;
        public AudioSource impactSource;

        [Header("Engine")]
        public AudioClip engineLoop;
        [Tooltip("Pitch range mapped to RPM (min at idle, max at redline)")]
        public float minPitch = 0.5f;
        public float maxPitch = 2.0f;

        [Header("Tires")]
        public AudioClip tireScreechLoop;
        [Tooltip("Slip angle threshold to start screech")]
        public float screechThreshold = 0.15f;

        [Header("Impact")]
        public AudioClip impactClip;

        private VehicleController _controller;

        private void Awake()
        {
            _controller = GetComponent<VehicleController>();
        }

        private void Update()
        {
            if (engineSource != null && engineLoop != null)
            {
                if (!engineSource.isPlaying)
                {
                    engineSource.clip = engineLoop;
                    engineSource.loop = true;
                    engineSource.Play();
                }

                float rpmNormalized = Mathf.InverseLerp(800f, 7000f, _controller.EngineRPM);
                engineSource.pitch = Mathf.Lerp(minPitch, maxPitch, rpmNormalized);
                engineSource.volume = Mathf.Lerp(0.3f, 1f, rpmNormalized);
            }
        }

        /// <summary>
        /// Called by VehicleController.OnCollisionEnter to play impact sound.
        /// </summary>
        public void PlayImpact(float intensity)
        {
            if (impactSource != null && impactClip != null)
            {
                impactSource.pitch = Random.Range(0.8f, 1.2f);
                impactSource.volume = Mathf.Clamp01(intensity);
                impactSource.PlayOneShot(impactClip);
            }
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

- [ ] **Step 3: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Vehicle/VehicleAudio.cs
git commit -m "feat: add placeholder vehicle audio (engine pitch, impact sounds)"
```

---

### Task 14: Camera System

**Files:**
- Create: `Assets/LonelyHighway/Scripts/Camera/VehicleCameraController.cs`
- Create: `Assets/LonelyHighway/Scripts/Camera/ChaseCamera.cs`
- Create: `Assets/LonelyHighway/Scripts/Camera/InteriorCamera.cs`
- Create: `Assets/LonelyHighway/Scripts/Camera/MirrorRenderer.cs`

- [ ] **Step 1: Write ChaseCamera**

```csharp
// Assets/LonelyHighway/Scripts/Camera/ChaseCamera.cs
using UnityEngine;

namespace LonelyHighway.Camera
{
    public class ChaseCamera : MonoBehaviour
    {
        public Transform target;
        public float distance = 6f;
        public float height = 2.5f;
        public float damping = 5f;
        public float lookAheadDistance = 3f;

        private void LateUpdate()
        {
            if (target == null) return;

            Vector3 desiredPosition = target.position
                - target.forward * distance
                + Vector3.up * height;

            transform.position = Vector3.Lerp(transform.position, desiredPosition, damping * Time.deltaTime);
            transform.LookAt(target.position + target.forward * lookAheadDistance + Vector3.up * 1f);
        }
    }
}
```

- [ ] **Step 2: Write InteriorCamera**

```csharp
// Assets/LonelyHighway/Scripts/Camera/InteriorCamera.cs
using LonelyHighway.Vehicle;
using UnityEngine;

namespace LonelyHighway.Camera
{
    public class InteriorCamera : MonoBehaviour
    {
        public Transform vehicleBody;
        public Vector3 seatOffset = new(0f, 1.1f, 0.3f);

        [Header("Head Bob")]
        public float bobIntensity = 0.02f;
        public float swayIntensity = 0.01f;

        private VehicleController _controller;
        private Vector3 _bobOffset;

        private void Start()
        {
            if (vehicleBody != null)
                _controller = vehicleBody.GetComponent<VehicleController>();
        }

        private void LateUpdate()
        {
            if (vehicleBody == null) return;

            // Base position inside car
            Vector3 basePos = vehicleBody.TransformPoint(seatOffset);

            // Head bob from suspension (simplified: use vertical velocity)
            var rb = vehicleBody.GetComponent<Rigidbody>();
            if (rb != null)
            {
                Vector3 localVel = vehicleBody.InverseTransformDirection(rb.linearVelocity);
                _bobOffset = Vector3.Lerp(_bobOffset,
                    new Vector3(localVel.x * swayIntensity, localVel.y * bobIntensity, 0f),
                    10f * Time.deltaTime);
            }

            transform.position = basePos + vehicleBody.TransformDirection(_bobOffset);
            transform.rotation = vehicleBody.rotation;
        }
    }
}
```

- [ ] **Step 3: Write MirrorRenderer**

```csharp
// Assets/LonelyHighway/Scripts/Camera/MirrorRenderer.cs
using UnityEngine;

namespace LonelyHighway.Camera
{
    public class MirrorRenderer : MonoBehaviour
    {
        public UnityEngine.Camera mirrorCamera;
        public RenderTexture mirrorTexture;

        [Header("Performance")]
        [Tooltip("Target mirror update rate in Hz")]
        public float targetFPS = 30f;

        private float _lastRenderTime;

        private void Start()
        {
            if (mirrorCamera != null && mirrorTexture != null)
            {
                mirrorCamera.targetTexture = mirrorTexture;
                mirrorCamera.enabled = false; // Manually render
            }
        }

        private void LateUpdate()
        {
            if (mirrorCamera == null) return;
            if (Time.time - _lastRenderTime < 1f / targetFPS) return;

            _lastRenderTime = Time.time;
            mirrorCamera.Render();
        }
    }
}
```

- [ ] **Step 4: Write VehicleCameraController**

```csharp
// Assets/LonelyHighway/Scripts/Camera/VehicleCameraController.cs
using LonelyHighway.Vehicle;
using UnityEngine;

namespace LonelyHighway.Camera
{
    public enum CameraMode
    {
        Interior,
        Hood,
        Chase,
        FreeLook
    }

    public class VehicleCameraController : MonoBehaviour
    {
        public VehicleInput vehicleInput;
        public UnityEngine.Camera mainCamera;

        [Header("Camera Rigs")]
        public InteriorCamera interiorRig;
        public ChaseCamera chaseRig;
        public Transform hoodCamAnchor;

        [Header("Mirrors (interior only)")]
        public MirrorRenderer[] mirrors;

        public CameraMode CurrentMode { get; private set; } = CameraMode.Chase;

        private void Update()
        {
            if (vehicleInput != null && vehicleInput.CycleCamera)
            {
                CurrentMode = (CameraMode)(((int)CurrentMode + 1) % 4);
                ApplyMode();
            }
        }

        private void ApplyMode()
        {
            // Disable all rigs
            if (interiorRig != null) interiorRig.enabled = false;
            if (chaseRig != null) chaseRig.enabled = false;

            // Enable mirrors only in interior mode
            foreach (var mirror in mirrors)
            {
                if (mirror != null) mirror.enabled = (CurrentMode == CameraMode.Interior);
            }

            switch (CurrentMode)
            {
                case CameraMode.Interior:
                    if (interiorRig != null)
                    {
                        interiorRig.enabled = true;
                        mainCamera.transform.SetParent(interiorRig.transform);
                        mainCamera.transform.localPosition = Vector3.zero;
                        mainCamera.transform.localRotation = Quaternion.identity;
                    }
                    break;

                case CameraMode.Hood:
                    if (hoodCamAnchor != null)
                    {
                        mainCamera.transform.SetParent(hoodCamAnchor);
                        mainCamera.transform.localPosition = Vector3.zero;
                        mainCamera.transform.localRotation = Quaternion.identity;
                    }
                    break;

                case CameraMode.Chase:
                    if (chaseRig != null)
                    {
                        chaseRig.enabled = true;
                        mainCamera.transform.SetParent(chaseRig.transform);
                        mainCamera.transform.localPosition = Vector3.zero;
                        mainCamera.transform.localRotation = Quaternion.identity;
                    }
                    break;

                case CameraMode.FreeLook:
                    mainCamera.transform.SetParent(chaseRig.target);
                    break;
            }
        }

        private void Start()
        {
            ApplyMode();
        }
    }
}
```

- [ ] **Step 5: Create camera assembly definition**

Create `Assets/LonelyHighway/Scripts/Camera/LonelyHighway.Camera.asmdef`:
```json
{
  "name": "LonelyHighway.Camera",
  "rootNamespace": "LonelyHighway.Camera",
  "references": ["LonelyHighway.Vehicle"],
  "includePlatforms": [],
  "autoReferenced": true
}
```

- [ ] **Step 6: Verify everything compiles**

- [ ] **Step 7: Wire up cameras in TestTrack scene**

Add camera rigs to the PlayerVehicle prefab:
1. Create child "ChaseCamRig" with ChaseCamera component, target = PlayerVehicle transform
2. Create child "InteriorCamRig" with InteriorCamera component, vehicleBody = PlayerVehicle transform
3. Create child "HoodCamAnchor" at position (0, 1.2, 2.0)
4. Add VehicleCameraController to PlayerVehicle, wire up all references
5. Test cycling cameras with C key in Play mode

- [ ] **Step 8: Commit**

```bash
git add Assets/LonelyHighway/Scripts/Camera/
git commit -m "feat: implement camera system with interior, hood, chase, and free look modes"
```

---

### Task 15: Tuning & Polish Pass

Final manual tuning on the test track.

- [ ] **Step 1: Tune vehicle profile**

In Play mode, adjust BYD-Qin VehicleProfile values until driving feels right:
- Suspension: car should settle in ~0.5s, no excessive bouncing
- Steering: responsive at low speed, stable at highway speed
- Brakes: should stop from 100km/h in ~40m (realistic)
- Gears: should shift smoothly in auto mode, reach ~120km/h in top gear

Document final values in a comment on the BYD-Qin asset.

- [ ] **Step 2: Test damage system**

Drive into a wall at various speeds:
- Low speed (<20km/h): no visible damage
- Medium speed (30-50km/h): minor panel damage, slight alignment drift
- High speed (>70km/h): major damage, engine stutter
- Wait 5 minutes: verify passive recovery works
- Verify damage doesn't make the car undrivable

- [ ] **Step 3: Test surface transitions**

Drive over the wet and painted line sections:
- Verify grip reduction is noticeable but not instant-spin
- Verify smooth transition between surfaces

- [ ] **Step 4: Commit final tuning**

```bash
git add -A
git commit -m "feat: tune vehicle physics — suspension, steering, brakes, damage thresholds"
```

---

## Summary

| Task | Component | Tests |
|------|-----------|-------|
| 1 | Unity project setup | — |
| 2 | Data definitions | — |
| 3 | Pacejka tire model | 6 unit tests |
| 4 | Suspension system | 6 unit tests |
| 5 | Drivetrain | 7 unit tests |
| 6 | Weight transfer | 5 unit tests |
| 7 | Steering system | 5 unit tests |
| 8 | Damage system | 7 unit tests |
| 9 | Wheel raycast + SurfaceIdentifier + Garage stub | — |
| 10 | Vehicle input (keyboard, gamepad, steering wheel) | — |
| 11 | Vehicle controller | — (wiring) |
| 12 | Prefab + integration + surface grip tests | 3 PlayMode tests |
| 13 | Vehicle audio | — (placeholder) |
| 14 | Camera system | — (manual) |
| 15 | Tuning & polish | — (manual) |

**Total: 15 tasks, ~36 unit tests, ~3 integration tests**

## Deferred to Later Plans

| Feature | Reason | When |
|---------|--------|------|
| Visual damage (mesh deformation, texture swaps) | Requires vehicle 3D model with separate panel meshes | After art pipeline / 3D model acquisition |
| Force feedback for steering wheels | Requires FMOD/audio plan for haptic integration | Environment plan |
| Garage fade-to-black UI | Requires UI system | Progression + UI plan |
| FreeLook camera (mouse orbit) | Non-essential, chase cam covers external view | Polish pass |
