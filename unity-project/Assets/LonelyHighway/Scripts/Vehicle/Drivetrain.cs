using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public struct DrivetrainState
    {
        public int currentGear;
        public float rpm;
    }

    public static class Drivetrain
    {
        public static float CalculateWheelTorque(float throttle, DrivetrainState state, VehicleProfile profile)
        {
            if (throttle <= 0f)
            {
                float brakingTorque = -profile.engineBraking * (state.rpm / profile.maxRPM);
                float gearRatio = state.currentGear >= 0
                    ? profile.gearRatios[state.currentGear]
                    : profile.reverseGearRatio;
                return brakingTorque * gearRatio * profile.finalDriveRatio;
            }

            float engineTorque = profile.torqueCurve.Evaluate(state.rpm) * throttle;
            float gr = state.currentGear >= 0
                ? profile.gearRatios[state.currentGear]
                : profile.reverseGearRatio;

            return engineTorque * gr * profile.finalDriveRatio;
        }

        public static float CalculateRPM(float wheelAngularVelocity, int gear, VehicleProfile profile)
        {
            float gearRatio = gear >= 0
                ? profile.gearRatios[gear]
                : profile.reverseGearRatio;

            float rpm = Mathf.Abs(wheelAngularVelocity) * gearRatio * profile.finalDriveRatio * 60f / (2f * Mathf.PI);
            return Mathf.Clamp(rpm, profile.idleRPM, profile.maxRPM);
        }

        public static (float left, float right) OpenDifferential(float totalWheelTorque, float leftAngVel, float rightAngVel)
        {
            return (totalWheelTorque / 2f, totalWheelTorque / 2f);
        }

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
