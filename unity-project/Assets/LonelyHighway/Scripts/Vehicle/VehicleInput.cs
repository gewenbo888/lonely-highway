using UnityEngine;
using UnityEngine.InputSystem;

namespace LonelyHighway.Vehicle
{
    public class VehicleInput : MonoBehaviour
    {
        public float Steer { get; private set; }
        public float Throttle { get; private set; }
        public float Brake { get; private set; }
        public bool Handbrake { get; private set; }
        public bool Horn { get; private set; }

        public bool ShiftUp { get; private set; }
        public bool ShiftDown { get; private set; }
        public bool HeadlightsToggle { get; private set; }
        public bool TurnSignalLeft { get; private set; }
        public bool TurnSignalRight { get; private set; }
        public bool WipersToggle { get; private set; }
        public bool CycleCamera { get; private set; }

        private VehicleInputActions _actions;

        private bool _shiftUpBuffer, _shiftDownBuffer;
        private bool _headlightsBuffer, _turnLeftBuffer, _turnRightBuffer;
        private bool _wipersBuffer, _cameraCycleBuffer;

        private void OnEnable()
        {
            _actions = new VehicleInputActions();
            _actions.Vehicle.Enable();
        }

        private void OnDisable()
        {
            _actions.Vehicle.Disable();
            _actions.Dispose();
        }

        private void Update()
        {
            var v = _actions.Vehicle;
            Steer = v.Steer.ReadValue<float>();
            Throttle = Mathf.Clamp01(v.Throttle.ReadValue<float>());
            Brake = Mathf.Clamp01(v.Brake.ReadValue<float>());
            Handbrake = v.Handbrake.IsPressed();
            Horn = v.Horn.IsPressed();

            _shiftUpBuffer |= v.ShiftUp.WasPressedThisFrame();
            _shiftDownBuffer |= v.ShiftDown.WasPressedThisFrame();
            _headlightsBuffer |= v.Headlights.WasPressedThisFrame();
            _turnLeftBuffer |= v.TurnSignalLeft.WasPressedThisFrame();
            _turnRightBuffer |= v.TurnSignalRight.WasPressedThisFrame();
            _wipersBuffer |= v.Wipers.WasPressedThisFrame();
            _cameraCycleBuffer |= v.CycleCamera.WasPressedThisFrame();
        }

        public void ConsumeBufferedInputs()
        {
            ShiftUp = _shiftUpBuffer;
            ShiftDown = _shiftDownBuffer;
            HeadlightsToggle = _headlightsBuffer;
            TurnSignalLeft = _turnLeftBuffer;
            TurnSignalRight = _turnRightBuffer;
            WipersToggle = _wipersBuffer;
            CycleCamera = _cameraCycleBuffer;

            _shiftUpBuffer = _shiftDownBuffer = false;
            _headlightsBuffer = _turnLeftBuffer = _turnRightBuffer = false;
            _wipersBuffer = _cameraCycleBuffer = false;
        }
    }
}
