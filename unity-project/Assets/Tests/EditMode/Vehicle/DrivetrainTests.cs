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
            _profile.engineBraking = 50f;
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
            Assert.Greater(torque, 1500f);
            Assert.Less(torque, 2500f);
        }

        [Test]
        public void WheelTorque_ZeroThrottle_ReturnsNegativeEngineBraking()
        {
            var state = new DrivetrainState { currentGear = 0, rpm = 4000f };
            float torque = Drivetrain.CalculateWheelTorque(0f, state, _profile);
            Assert.Less(torque, 0f, "Engine braking should produce negative torque");
        }

        [Test]
        public void RPMFromWheelSpeed_FirstGear_CalculatesCorrectly()
        {
            float rpm = Drivetrain.CalculateRPM(50f, 0, _profile);
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
