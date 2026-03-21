using NUnit.Framework;
using LonelyHighway.Vehicle;

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
            Assert.AreEqual(1750f, force, 0.01f);
        }

        [Test]
        public void DamperForce_CompressingVelocity_AddsForce()
        {
            float forceStatic = SuspensionSystem.CalculateSpringForce(
                compression: 0.05f, velocity: 0f,
                springRate: 35000f, damperRate: 4500f);
            float forceDynamic = SuspensionSystem.CalculateSpringForce(
                compression: 0.05f, velocity: -0.5f,
                springRate: 35000f, damperRate: 4500f);
            Assert.Greater(forceDynamic, forceStatic);
        }

        [Test]
        public void SpringForce_FullyExtended_ReturnsZero()
        {
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
            Assert.Greater(force, 0f);
            Assert.AreEqual(300f, force, 0.01f);
        }
    }
}
