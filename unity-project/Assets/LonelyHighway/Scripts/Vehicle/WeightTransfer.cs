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
        public static WheelLoads CalculateWheelLoads(
            float mass, float wheelbase, float trackWidth, float comHeight,
            float frontAxleOffset, float longitudinalAccel, float lateralAccel)
        {
            float gravity = 9.81f;
            float totalWeight = mass * gravity;

            float rearAxleOffset = wheelbase - frontAxleOffset;

            float staticFrontTotal = totalWeight * rearAxleOffset / wheelbase;
            float staticRearTotal = totalWeight * frontAxleOffset / wheelbase;

            float longTransfer = mass * longitudinalAccel * comHeight / wheelbase;

            float frontTotal = staticFrontTotal - longTransfer;
            float rearTotal = staticRearTotal + longTransfer;

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
