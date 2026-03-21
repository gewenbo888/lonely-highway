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
            _profile.suspensionSagRate = 0.3f;
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

            state.ApplyPassiveRecovery(60f, _profile);
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
