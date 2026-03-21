using UnityEngine;

namespace LonelyHighway.Camera
{
    public class MirrorRenderer : MonoBehaviour
    {
        public UnityEngine.Camera mirrorCamera;
        public RenderTexture mirrorTexture;

        [Header("Performance")]
        public float targetFPS = 30f;

        private float _lastRenderTime;

        private void Start()
        {
            if (mirrorCamera != null && mirrorTexture != null)
            {
                mirrorCamera.targetTexture = mirrorTexture;
                mirrorCamera.enabled = false;
            }
        }

        private void LateUpdate()
        {
            if (mirrorCamera == null) return;
            if (Time.time - _lastRenderTime < 1f / targetFPS) return;

            _lastRenderTime = Time.time;
            mirrorCamera.Render();
        }
    }
}
