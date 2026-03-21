using LonelyHighway.Data;
using UnityEngine;
using System.Collections.Generic;

namespace LonelyHighway.Vehicle
{
    public class VehicleController : MonoBehaviour
    {
        [Header("Configuration")]
        public VehicleProfile profile;
        public TireProfile defaultTireProfile;
        public List<SurfaceTireMapping> surfaceTireMappings = new();

        [Header("Wheel Transforms (set in inspector)")]
        public WheelRaycast wheelFL;
        public WheelRaycast wheelFR;
        public WheelRaycast wheelRL;
        public WheelRaycast wheelRR;

        [Header("Wheel Mesh Transforms (visual only)")]
        public Transform wheelMeshFL;
        public Transform wheelMeshFR;
        public Transform wheelMeshRL;
        public Transform wheelMeshRR;

        public float SpeedKmh => _rb.linearVelocity.magnitude * 3.6f;
        public float EngineRPM => _drivetrainState.rpm;
        public int CurrentGear => _drivetrainState.currentGear;
        public DamageState Damage => _damageState;

        private Rigidbody _rb;
        private VehicleInput _input;
        private DrivetrainState _drivetrainState;
        private DamageState _damageState;
        private bool _autoTransmission = true;

        private float[] _wheelAngularVelocity = new float[4];
        private Vector3 _previousLocalVelocity;

        [System.Serializable]
        public struct SurfaceTireMapping
        {
            public SurfaceType surface;
            public TireProfile tireProfile;
        }

        private void Awake()
        {
            _rb = GetComponent<Rigidbody>();
            _input = GetComponent<VehicleInput>();
            _damageState = new DamageState();
            _drivetrainState = new DrivetrainState { currentGear = 0, rpm = profile.idleRPM };

            _rb.mass = profile.mass;
            _rb.centerOfMass = profile.centerOfMass;
        }

        private void FixedUpdate()
        {
            _input.ConsumeBufferedInputs();
            float dt = Time.fixedDeltaTime;

            var hits = new WheelHitInfo[4];
            WheelRaycast[] wheels = { wheelFL, wheelFR, wheelRL, wheelRR };
            DamagePanel[] wheelPanels = { DamagePanel.FrontLeft, DamagePanel.FrontRight, DamagePanel.RearLeft, DamagePanel.RearRight };
            for (int i = 0; i < 4; i++)
            {
                float sagMultiplier = _damageState.GetSuspensionSagMultiplier(wheelPanels[i], profile);
                float effectiveRestLength = profile.restLength * sagMultiplier;
                hits[i] = wheels[i].CastWheel(effectiveRestLength, profile.maxTravel, profile.wheelRadius);
            }

            float[] compressions = new float[4];
            for (int i = 0; i < 4; i++)
            {
                if (!hits[i].isGrounded) continue;

                compressions[i] = hits[i].compression;
                float springForce = SuspensionSystem.CalculateSpringForce(
                    hits[i].compression, hits[i].compressionVelocity,
                    profile.springRate, profile.damperRate);

                _rb.AddForceAtPosition(
                    hits[i].contactNormal * springForce,
                    wheels[i].transform.position);
            }

            float antiRollFront = SuspensionSystem.CalculateAntiRollForce(
                compressions[0], compressions[1], profile.antiRollBarStiffness);
            float antiRollRear = SuspensionSystem.CalculateAntiRollForce(
                compressions[2], compressions[3], profile.antiRollBarStiffness);

            if (hits[0].isGrounded)
                _rb.AddForceAtPosition(transform.up * antiRollFront, wheels[0].transform.position);
            if (hits[1].isGrounded)
                _rb.AddForceAtPosition(-transform.up * antiRollFront, wheels[1].transform.position);
            if (hits[2].isGrounded)
                _rb.AddForceAtPosition(transform.up * antiRollRear, wheels[2].transform.position);
            if (hits[3].isGrounded)
                _rb.AddForceAtPosition(-transform.up * antiRollRear, wheels[3].transform.position);

            Vector3 localVelocity = transform.InverseTransformDirection(_rb.linearVelocity);
            Vector3 localAccel = (localVelocity - _previousLocalVelocity) / dt;
            _previousLocalVelocity = localVelocity;

            var wheelLoads = WeightTransfer.CalculateWheelLoads(
                profile.mass,
                profile.frontAxleOffset + profile.rearAxleOffset,
                profile.trackHalfWidth * 2f,
                profile.centerOfMass.y,
                profile.frontAxleOffset,
                localAccel.z, localAccel.x);
            float[] loads = { wheelLoads.frontLeft, wheelLoads.frontRight, wheelLoads.rearLeft, wheelLoads.rearRight };

            float steerAngle = SteeringSystem.CalculateSteerAngle(
                _input.Steer, _rb.linearVelocity.magnitude, profile);
            steerAngle += _damageState.GetAlignmentDrift(profile);

            float speed = _rb.linearVelocity.magnitude;
            for (int i = 0; i < 4; i++)
            {
                if (!hits[i].isGrounded) continue;

                TireProfile tireProfile = GetTireProfile(hits[i].surfaceType);

                Vector3 wheelWorldForward = wheels[i].transform.forward;
                Vector3 wheelWorldRight = wheels[i].transform.right;

                if (i < 2)
                {
                    wheelWorldForward = Quaternion.AngleAxis(steerAngle, transform.up) * wheels[i].transform.forward;
                    wheelWorldRight = Quaternion.AngleAxis(steerAngle, transform.up) * wheels[i].transform.right;
                }

                Vector3 pointVelocity = _rb.GetPointVelocity(wheels[i].transform.position);
                float forwardSpeed = Vector3.Dot(pointVelocity, wheelWorldForward);
                float sidewaysSpeed = Vector3.Dot(pointVelocity, wheelWorldRight);

                float slipAngle = (speed > 0.5f) ? Mathf.Atan2(sidewaysSpeed, Mathf.Abs(forwardSpeed)) : 0f;

                float wheelSpeed = _wheelAngularVelocity[i] * profile.wheelRadius;
                float slipRatio = (speed > 0.5f)
                    ? (wheelSpeed - forwardSpeed) / Mathf.Max(Mathf.Abs(forwardSpeed), Mathf.Abs(wheelSpeed), 0.5f)
                    : 0f;

                float lateralForce = PacejkaTireModel.CalculateLateralForce(slipAngle, loads[i], tireProfile);
                float longitudinalForce = PacejkaTireModel.CalculateLongitudinalForce(slipRatio, loads[i], tireProfile);

                Vector3 totalForce = wheelWorldForward * longitudinalForce - wheelWorldRight * lateralForce;
                _rb.AddForceAtPosition(totalForce, hits[i].contactPoint);

                float driveTorque = 0f;
                if (i >= 2)
                {
                    float totalAxleTorque = Drivetrain.CalculateWheelTorque(_input.Throttle, _drivetrainState, profile);
                    var (leftT, rightT) = Drivetrain.OpenDifferential(
                        totalAxleTorque, _wheelAngularVelocity[2], _wheelAngularVelocity[3]);
                    driveTorque = (i == 2) ? leftT : rightT;
                }

                float brakeTorque = _input.Brake * profile.maxBrakeForce *
                    (i < 2 ? profile.brakeBias : 1f - profile.brakeBias);

                if (_input.Handbrake && i >= 2)
                    brakeTorque += profile.handbrakeForce;

                float netTorque = driveTorque - Mathf.Sign(_wheelAngularVelocity[i]) * brakeTorque;
                float wheelInertia = profile.wheelMass * profile.wheelRadius * profile.wheelRadius;
                _wheelAngularVelocity[i] += (netTorque / wheelInertia) * dt;
            }

            float avgRearWheelAngVel = (_wheelAngularVelocity[2] + _wheelAngularVelocity[3]) / 2f;
            _drivetrainState.rpm = Drivetrain.CalculateRPM(avgRearWheelAngVel, _drivetrainState.currentGear, profile);

            if (_autoTransmission)
            {
                _drivetrainState.currentGear = Drivetrain.AutoShift(_drivetrainState, profile);
            }

            if (_input.ShiftUp && _drivetrainState.currentGear < profile.gearRatios.Length - 1)
            {
                _drivetrainState.currentGear++;
                _autoTransmission = false;
            }
            if (_input.ShiftDown && _drivetrainState.currentGear > -1)
            {
                _drivetrainState.currentGear--;
                _autoTransmission = false;
            }

            _damageState.ApplyPassiveRecovery(dt, profile);
            UpdateWheelMeshes(steerAngle);
        }

        private void OnCollisionEnter(Collision collision)
        {
            float impulse = collision.impulse.magnitude / 1000f;
            Vector3 localContact = transform.InverseTransformPoint(collision.GetContact(0).point);
            DamagePanel panel = DeterminePanel(localContact);
            _damageState.ApplyImpact(panel, impulse, profile);
        }

        private DamagePanel DeterminePanel(Vector3 localPoint)
        {
            bool isFront = localPoint.z > 0f;
            bool isLeft = localPoint.x < 0f;
            bool isCenter = Mathf.Abs(localPoint.x) < 0.3f;

            if (isCenter)
                return isFront ? DamagePanel.Front : DamagePanel.Rear;
            if (isFront)
                return isLeft ? DamagePanel.FrontLeft : DamagePanel.FrontRight;
            else
                return isLeft ? DamagePanel.RearLeft : DamagePanel.RearRight;
        }

        private TireProfile GetTireProfile(SurfaceType surface)
        {
            foreach (var mapping in surfaceTireMappings)
            {
                if (mapping.surface == surface)
                    return mapping.tireProfile;
            }
            return defaultTireProfile;
        }

        private void UpdateWheelMeshes(float steerAngle)
        {
            Transform[] meshes = { wheelMeshFL, wheelMeshFR, wheelMeshRL, wheelMeshRR };
            WheelRaycast[] wheels = { wheelFL, wheelFR, wheelRL, wheelRR };

            for (int i = 0; i < 4; i++)
            {
                if (meshes[i] == null) continue;

                Vector3 pos = wheels[i].transform.position - wheels[i].transform.up *
                    (profile.restLength - wheels[i].hitInfo.compression + profile.wheelRadius);

                if (!wheels[i].hitInfo.isGrounded)
                    pos = wheels[i].transform.position - wheels[i].transform.up *
                        (profile.restLength + profile.maxTravel + profile.wheelRadius);

                meshes[i].position = pos;

                if (i < 2)
                {
                    meshes[i].localRotation = Quaternion.Euler(0f, steerAngle, 0f);
                }

                float spinDegrees = _wheelAngularVelocity[i] * Mathf.Rad2Deg * Time.fixedDeltaTime;
                meshes[i].Rotate(Vector3.right, spinDegrees, Space.Self);
            }
        }
    }
}
