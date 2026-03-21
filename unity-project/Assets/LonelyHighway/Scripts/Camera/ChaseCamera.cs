using UnityEngine;

namespace LonelyHighway.Camera
{
    public class ChaseCamera : MonoBehaviour
    {
        public Transform target;
        public float distance = 6f;
        public float height = 2.5f;
        public float damping = 5f;
        public float lookAheadDistance = 3f;

        private void LateUpdate()
        {
            if (target == null) return;

            Vector3 desiredPosition = target.position
                - target.forward * distance
                + Vector3.up * height;

            transform.position = Vector3.Lerp(transform.position, desiredPosition, damping * Time.deltaTime);
            transform.LookAt(target.position + target.forward * lookAheadDistance + Vector3.up * 1f);
        }
    }
}
