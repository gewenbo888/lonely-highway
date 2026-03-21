using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public enum DamagePanel
    {
        FrontLeft,
        FrontRight,
        RearLeft,
        RearRight,
        Front,
        Rear
    }

    public class DamageState
    {
        private readonly float[] _panelHealth;
        private const int PanelCount = 6;

        public DamageState()
        {
            _panelHealth = new float[PanelCount];
            FullRepair();
        }

        public float GetPanelHealth(DamagePanel panel)
        {
            return _panelHealth[(int)panel];
        }

        public void ApplyImpact(DamagePanel panel, float impactForce, VehicleProfile profile)
        {
            if (impactForce <= profile.collisionDamageThreshold)
                return;

            float damage = (impactForce - profile.collisionDamageThreshold) * 2f;
            _panelHealth[(int)panel] = Mathf.Max(0f, _panelHealth[(int)panel] - damage);
        }

        public float GetAlignmentDrift(VehicleProfile profile)
        {
            float leftDamage = 100f - _panelHealth[(int)DamagePanel.FrontLeft];
            float rightDamage = 100f - _panelHealth[(int)DamagePanel.FrontRight];
            return (leftDamage - rightDamage) * profile.alignmentDriftRate / 100f;
        }

        public float GetEngineStutterFactor(VehicleProfile profile)
        {
            float avgFrontDamage = ((100f - _panelHealth[(int)DamagePanel.FrontLeft])
                + (100f - _panelHealth[(int)DamagePanel.FrontRight])
                + (100f - _panelHealth[(int)DamagePanel.Front])) / 300f;
            return avgFrontDamage * profile.engineStutterChance;
        }

        public float GetSuspensionSagMultiplier(DamagePanel panel, VehicleProfile profile)
        {
            float damage = (100f - _panelHealth[(int)panel]) / 100f;
            return 1f - (damage * profile.suspensionSagRate);
        }

        public void ApplyPassiveRecovery(float deltaTime, VehicleProfile profile)
        {
            float recovery = profile.passiveRecoveryRate * deltaTime;
            for (int i = 0; i < PanelCount; i++)
            {
                _panelHealth[i] = Mathf.Min(100f, _panelHealth[i] + recovery);
            }
        }

        public void FullRepair()
        {
            for (int i = 0; i < PanelCount; i++)
                _panelHealth[i] = 100f;
        }
    }
}
