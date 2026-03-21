namespace LonelyHighway.Vehicle
{
    public static class SuspensionSystem
    {
        public static float CalculateSpringForce(float compression, float velocity, float springRate, float damperRate)
        {
            if (compression <= 0f)
                return 0f;

            float spring = springRate * compression;
            float damper = damperRate * -velocity;
            float total = spring + damper;

            return total > 0f ? total : 0f;
        }

        public static float CalculateAntiRollForce(float leftCompression, float rightCompression, float stiffness)
        {
            return stiffness * (leftCompression - rightCompression);
        }
    }
}
