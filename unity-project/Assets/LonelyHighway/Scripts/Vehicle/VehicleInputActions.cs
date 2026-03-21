// Stub for auto-generated Input Actions class
// Replace with Unity-generated version when project is opened in Editor
using System;
using UnityEngine.InputSystem;

namespace LonelyHighway.Vehicle
{
    public class VehicleInputActions : IDisposable
    {
        public VehicleActionMap Vehicle;

        public VehicleInputActions()
        {
            Vehicle = new VehicleActionMap();
        }

        public void Dispose() { }

        public class VehicleActionMap
        {
            public InputAction Steer = new InputAction("Steer");
            public InputAction Throttle = new InputAction("Throttle");
            public InputAction Brake = new InputAction("Brake");
            public InputAction Handbrake = new InputAction("Handbrake");
            public InputAction ShiftUp = new InputAction("ShiftUp");
            public InputAction ShiftDown = new InputAction("ShiftDown");
            public InputAction Horn = new InputAction("Horn");
            public InputAction Headlights = new InputAction("Headlights");
            public InputAction TurnSignalLeft = new InputAction("TurnSignalLeft");
            public InputAction TurnSignalRight = new InputAction("TurnSignalRight");
            public InputAction Wipers = new InputAction("Wipers");
            public InputAction CycleCamera = new InputAction("CycleCamera");

            public void Enable()
            {
                Steer.Enable(); Throttle.Enable(); Brake.Enable();
                Handbrake.Enable(); ShiftUp.Enable(); ShiftDown.Enable();
                Horn.Enable(); Headlights.Enable(); TurnSignalLeft.Enable();
                TurnSignalRight.Enable(); Wipers.Enable(); CycleCamera.Enable();
            }

            public void Disable()
            {
                Steer.Disable(); Throttle.Disable(); Brake.Disable();
                Handbrake.Disable(); ShiftUp.Disable(); ShiftDown.Disable();
                Horn.Disable(); Headlights.Disable(); TurnSignalLeft.Disable();
                TurnSignalRight.Disable(); Wipers.Disable(); CycleCamera.Disable();
            }
        }
    }
}
