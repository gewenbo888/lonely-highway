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
            Assert.AreEqual(10.5f, angle, 0.1f);
        }

        [Test]
        public void SteerAngle_AtHalfSpeed_ReturnsMidAngle()
        {
            float angle = SteeringSystem.CalculateSteerAngle(1f, 15f, _profile);
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
            Assert.Greater(torque, 0f);
        }
    }
}
