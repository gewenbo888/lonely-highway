using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public static class PacejkaTireModel
    {
        private static float MagicFormula(float slip, float normalLoad, float B, float C, float D, float E)
        {
            float x = slip;
            float Bx = B * x;
            float force = normalLoad * D * Mathf.Sin(C * Mathf.Atan(Bx - E * (Bx - Mathf.Atan(Bx))));
            return force;
        }

        public static float CalculateLateralForce(float slipAngle, float normalLoad, TireProfile profile)
        {
            return MagicFormula(slipAngle, normalLoad,
                profile.lateralB, profile.lateralC, profile.lateralD, profile.lateralE);
        }

        public static float CalculateLongitudinalForce(float slipRatio, float normalLoad, TireProfile profile)
        {
            return MagicFormula(slipRatio, normalLoad,
                profile.longitudinalB, profile.longitudinalC, profile.longitudinalD, profile.longitudinalE);
        }
    }
}
