using LonelyHighway.Vehicle;
using UnityEngine;

namespace LonelyHighway.Camera
{
    public enum CameraMode
    {
        Interior,
        Hood,
        Chase,
        FreeLook
    }

    public class VehicleCameraController : MonoBehaviour
    {
        public VehicleInput vehicleInput;
        public UnityEngine.Camera mainCamera;

        [Header("Camera Rigs")]
        public InteriorCamera interiorRig;
        public ChaseCamera chaseRig;
        public Transform hoodCamAnchor;

        [Header("Mirrors (interior only)")]
        public MirrorRenderer[] mirrors;

        public CameraMode CurrentMode { get; private set; } = CameraMode.Chase;

        private void Update()
        {
            if (vehicleInput != null && vehicleInput.CycleCamera)
            {
                CurrentMode = (CameraMode)(((int)CurrentMode + 1) % 4);
                ApplyMode();
            }
        }

        private void ApplyMode()
        {
            if (interiorRig != null) interiorRig.enabled = false;
            if (chaseRig != null) chaseRig.enabled = false;

            foreach (var mirror in mirrors)
            {
                if (mirror != null) mirror.enabled = (CurrentMode == CameraMode.Interior);
            }

            switch (CurrentMode)
            {
                case CameraMode.Interior:
                    if (interiorRig != null)
                    {
                        interiorRig.enabled = true;
                        mainCamera.transform.SetParent(interiorRig.transform);
                        mainCamera.transform.localPosition = Vector3.zero;
                        mainCamera.transform.localRotation = Quaternion.identity;
                    }
                    break;

                case CameraMode.Hood:
                    if (hoodCamAnchor != null)
                    {
                        mainCamera.transform.SetParent(hoodCamAnchor);
                        mainCamera.transform.localPosition = Vector3.zero;
                        mainCamera.transform.localRotation = Quaternion.identity;
                    }
                    break;

                case CameraMode.Chase:
                    if (chaseRig != null)
                    {
                        chaseRig.enabled = true;
                        mainCamera.transform.SetParent(chaseRig.transform);
                        mainCamera.transform.localPosition = Vector3.zero;
                        mainCamera.transform.localRotation = Quaternion.identity;
                    }
                    break;

                case CameraMode.FreeLook:
                    if (chaseRig != null && chaseRig.target != null)
                        mainCamera.transform.SetParent(chaseRig.target);
                    break;
            }
        }

        private void Start()
        {
            ApplyMode();
        }
    }
}
