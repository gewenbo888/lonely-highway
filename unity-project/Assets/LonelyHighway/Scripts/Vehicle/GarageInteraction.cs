using UnityEngine;

namespace LonelyHighway.Vehicle
{
    [RequireComponent(typeof(BoxCollider))]
    public class GarageInteraction : MonoBehaviour
    {
        private void Awake()
        {
            var col = GetComponent<BoxCollider>();
            col.isTrigger = true;
        }

        private void OnTriggerEnter(Collider other)
        {
            var controller = other.GetComponentInParent<VehicleController>();
            if (controller == null) return;
            controller.Damage.FullRepair();
            Debug.Log("Vehicle repaired at garage");
        }
    }
}
