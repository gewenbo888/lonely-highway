using LonelyHighway.Data;
using UnityEngine;

namespace LonelyHighway.Vehicle
{
    public struct WheelHitInfo
    {
        public bool isGrounded;
        public float compression;
        public float compressionVelocity;
        public Vector3 contactPoint;
        public Vector3 contactNormal;
        public SurfaceType surfaceType;
        public Collider hitCollider;
    }

    public class WheelRaycast : MonoBehaviour
    {
        [HideInInspector] public WheelHitInfo hitInfo;
        private float _previousCompression;

        public WheelHitInfo CastWheel(float restLength, float maxTravel, float wheelRadius)
        {
            float rayLength = restLength + maxTravel + wheelRadius;
            var origin = transform.position;

            if (Physics.Raycast(origin, -transform.up, out RaycastHit hit, rayLength))
            {
                float springLength = hit.distance - wheelRadius;
                float compression = restLength - springLength;
                float compressionVelocity = (compression - _previousCompression) / Time.fixedDeltaTime;
                _previousCompression = compression;

                SurfaceType surface = SurfaceType.DryAsphalt;
                var surfaceId = hit.collider.GetComponent<SurfaceIdentifier>();
                if (surfaceId != null) surface = surfaceId.surfaceType;

                hitInfo = new WheelHitInfo
                {
                    isGrounded = true,
                    compression = compression,
                    compressionVelocity = compressionVelocity,
                    contactPoint = hit.point,
                    contactNormal = hit.normal,
                    surfaceType = surface,
                    hitCollider = hit.collider
                };
            }
            else
            {
                _previousCompression = 0f;
                hitInfo = new WheelHitInfo { isGrounded = false };
            }
            return hitInfo;
        }
    }
}
