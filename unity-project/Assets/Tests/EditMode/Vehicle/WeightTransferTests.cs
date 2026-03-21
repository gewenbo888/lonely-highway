using NUnit.Framework;
using LonelyHighway.Vehicle;
using UnityEngine;

namespace LonelyHighway.Tests.EditMode
{
    public class WeightTransferTests
    {
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
            var loads = WeightTransfer.CalculateWheelLoads(
                _mass, _wheelbase, _trackWidth, _comHeight, _frontAxleOffset,
                longitudinalAccel: -8f, lateralAccel: 8f);

            float totalWeight = _mass * 9.81f;
            float totalLoad = loads.frontLeft + loads.frontRight + loads.rearLeft + loads.rearRight;
            Assert.LessOrEqual(totalLoad, totalWeight + 1f);
        }
    }
}
