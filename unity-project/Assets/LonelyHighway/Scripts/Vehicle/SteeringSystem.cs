using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public static class SteeringSystem
    {
        public static float CalculateSteerAngle(float steerInput, float speed, VehicleProfile profile)
        {
            float t = Mathf.Clamp01(speed / profile.steerLimitSpeed);
            float maxAngle = Mathf.Lerp(profile.maxSteerAngle, profile.maxSteerAngle * profile.highSpeedSteerMultiplier, t);
            return steerInput * maxAngle;
        }

        public static float CalculateSelfAlignTorque(float slipAngle, float normalLoad, float trailLength)
        {
            return normalLoad * trailLength * Mathf.Sin(slipAngle);
        }
    }
}
