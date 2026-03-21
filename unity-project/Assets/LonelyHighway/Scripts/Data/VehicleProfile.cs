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
