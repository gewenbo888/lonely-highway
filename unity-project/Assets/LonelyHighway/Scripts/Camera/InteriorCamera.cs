using LonelyHighway.Vehicle;
using UnityEngine;

namespace LonelyHighway.Camera
{
    public class InteriorCamera : MonoBehaviour
    {
        public Transform vehicleBody;
        public Vector3 seatOffset = new(0f, 1.1f, 0.3f);

        [Header("Head Bob")]
        public float bobIntensity = 0.02f;
        public float swayIntensity = 0.01f;

        private VehicleController _controller;
        private Vector3 _bobOffset;

        private void Start()
        {
            if (vehicleBody != null)
                _controller = vehicleBody.GetComponent<VehicleController>();
        }

        private void LateUpdate()
        {
            if (vehicleBody == null) return;

            Vector3 basePos = vehicleBody.TransformPoint(seatOffset);

            var rb = vehicleBody.GetComponent<Rigidbody>();
            if (rb != null)
            {
                Vector3 localVel = vehicleBody.InverseTransformDirection(rb.linearVelocity);
                _bobOffset = Vector3.Lerp(_bobOffset,
                    new Vector3(localVel.x * swayIntensity, localVel.y * bobIntensity, 0f),
                    10f * Time.deltaTime);
            }

            transform.position = basePos + vehicleBody.TransformDirection(_bobOffset);
            transform.rotation = vehicleBody.rotation;
        }
    }
}
