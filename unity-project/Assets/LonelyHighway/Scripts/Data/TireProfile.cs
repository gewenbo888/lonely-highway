using UnityEngine;

namespace LonelyHighway.Data
{
    [CreateAssetMenu(fileName = "NewTireProfile", menuName = "LonelyHighway/Tire Profile")]
    public class TireProfile : ScriptableObject
    {
        [Header("Pacejka Lateral (Fy) Coefficients")]
        [Tooltip("Peak factor")] public float lateralB = 10f;
        [Tooltip("Shape factor")] public float lateralC = 1.9f;
        [Tooltip("Peak value")] public float lateralD = 1.0f;
        [Tooltip("Curvature")] public float lateralE = -0.97f;

        [Header("Pacejka Longitudinal (Fx) Coefficients")]
        [Tooltip("Peak factor")] public float longitudinalB = 12f;
        [Tooltip("Shape factor")] public float longitudinalC = 2.3f;
        [Tooltip("Peak value")] public float longitudinalD = 1.0f;
        [Tooltip("Curvature")] public float longitudinalE = -0.96f;
    }
}
