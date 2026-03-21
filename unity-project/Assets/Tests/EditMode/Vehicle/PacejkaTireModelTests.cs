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
