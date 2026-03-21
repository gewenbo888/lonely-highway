using UnityEngine;

namespace LonelyHighway.Vehicle
{
    [RequireComponent(typeof(VehicleController))]
    public class VehicleAudio : MonoBehaviour
    {
        [Header("Audio Sources")]
        public AudioSource engineSource;
        public AudioSource tireSource;
        public AudioSource impactSource;

        [Header("Engine")]
        public AudioClip engineLoop;
        public float minPitch = 0.5f;
        public float maxPitch = 2.0f;

        [Header("Tires")]
        public AudioClip tireScreechLoop;
        public float screechThreshold = 0.15f;

        [Header("Impact")]
        public AudioClip impactClip;

        private VehicleController _controller;

        private void Awake()
        {
            _controller = GetComponent<VehicleController>();
        }

        private void Update()
        {
            if (engineSource != null && engineLoop != null)
            {
                if (!engineSource.isPlaying)
                {
                    engineSource.clip = engineLoop;
                    engineSource.loop = true;
                    engineSource.Play();
                }

                float rpmNormalized = Mathf.InverseLerp(800f, 7000f, _controller.EngineRPM);
                engineSource.pitch = Mathf.Lerp(minPitch, maxPitch, rpmNormalized);
                engineSource.volume = Mathf.Lerp(0.3f, 1f, rpmNormalized);
            }
        }

        public void PlayImpact(float intensity)
        {
            if (impactSource != null && impactClip != null)
            {
                impactSource.pitch = Random.Range(0.8f, 1.2f);
                impactSource.volume = Mathf.Clamp01(intensity);
                impactSource.PlayOneShot(impactClip);
            }
        }
    }
}
